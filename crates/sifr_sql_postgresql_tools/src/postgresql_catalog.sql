WITH user_namespaces AS (
    SELECT oid, nspname
    FROM pg_catalog.pg_namespace
    WHERE nspname <> 'information_schema'
      AND nspname NOT LIKE 'pg\_%' ESCAPE '\'
),
relations AS (
    SELECT c.oid, n.nspname, c.relname, c.relkind,
           n.nspname || '.' || c.relname AS identity
    FROM pg_catalog.pg_class c
    JOIN user_namespaces n ON n.oid = c.relnamespace
    WHERE c.relkind IN ('r', 'p', 'v', 'm')
),
catalog_objects AS (
    SELECT n.nspname AS identity, 'namespace' AS object_kind,
           jsonb_build_object('name', n.nspname) AS semantic,
           '[]'::jsonb AS dependencies
    FROM user_namespaces n

    UNION ALL
    SELECT r.identity,
           CASE r.relkind WHEN 'v' THEN 'view' WHEN 'm' THEN 'materialized-view' ELSE 'table' END,
           jsonb_strip_nulls(jsonb_build_object(
               'name', r.relname,
               'relation-kind', r.relkind,
               'definition', CASE WHEN r.relkind IN ('v', 'm') THEN pg_catalog.pg_get_viewdef(r.oid, true) END,
               'columns', COALESCE((
                   SELECT jsonb_agg(r.identity || '.' || a.attname ORDER BY a.attnum)
                   FROM pg_catalog.pg_attribute a
                   WHERE a.attrelid = r.oid AND a.attnum > 0 AND NOT a.attisdropped
               ), '[]'::jsonb)
           )), jsonb_build_array(r.nspname)
    FROM relations r

    UNION ALL
    SELECT r.identity || '.' || a.attname, 'column',
           jsonb_build_object(
               'name', a.attname,
               'database-type-name', pg_catalog.format_type(a.atttypid, a.atttypmod),
               'nullable', NOT a.attnotnull,
               'has-default', a.atthasdef,
               'generated', a.attgenerated <> '',
               'identity', a.attidentity,
               'position', a.attnum,
               'default-expression', COALESCE(pg_catalog.pg_get_expr(d.adbin, d.adrelid, true), '')
           ), jsonb_build_array(r.identity)
    FROM relations r
    JOIN pg_catalog.pg_attribute a ON a.attrelid = r.oid
    LEFT JOIN pg_catalog.pg_attrdef d ON d.adrelid = a.attrelid AND d.adnum = a.attnum
    WHERE a.attnum > 0 AND NOT a.attisdropped

    UNION ALL
    SELECT r.identity || '._constraint_' || con.conname,
           CASE con.contype WHEN 'p' THEN 'primary-key' WHEN 'u' THEN 'unique-constraint'
                WHEN 'f' THEN 'foreign-key' ELSE 'check-constraint' END,
           jsonb_build_object('name', con.conname, 'definition', pg_catalog.pg_get_constraintdef(con.oid, true),
                              'validated', con.convalidated, 'deferrable', con.condeferrable,
                              'initially-deferred', con.condeferred),
           CASE WHEN con.contype = 'f' AND target.oid IS NOT NULL
                THEN jsonb_build_array(r.identity, target_n.nspname || '.' || target.relname)
                ELSE jsonb_build_array(r.identity) END
    FROM pg_catalog.pg_constraint con
    JOIN relations r ON r.oid = con.conrelid
    LEFT JOIN pg_catalog.pg_class target ON target.oid = con.confrelid
    LEFT JOIN pg_catalog.pg_namespace target_n ON target_n.oid = target.relnamespace
    WHERE con.contype IN ('p', 'u', 'f', 'c')

    UNION ALL
    SELECT r.identity || '._index_' || i.relname, 'index',
           jsonb_build_object('name', i.relname, 'definition', pg_catalog.pg_get_indexdef(i.oid),
                              'unique', x.indisunique, 'valid', x.indisvalid, 'ready', x.indisready),
           jsonb_build_array(r.identity)
    FROM pg_catalog.pg_index x
    JOIN pg_catalog.pg_class i ON i.oid = x.indexrelid
    JOIN relations r ON r.oid = x.indrelid

    UNION ALL
    SELECT n.nspname || '.' || c.relname, 'sequence',
           jsonb_build_object('name', c.relname, 'data-type', pg_catalog.format_type(s.seqtypid, NULL),
                              'start', s.seqstart, 'increment', s.seqincrement, 'minimum', s.seqmin,
                              'maximum', s.seqmax, 'cache', s.seqcache, 'cycle', s.seqcycle),
           jsonb_build_array(n.nspname)
    FROM pg_catalog.pg_sequence s
    JOIN pg_catalog.pg_class c ON c.oid = s.seqrelid
    JOIN user_namespaces n ON n.oid = c.relnamespace

    UNION ALL
    SELECT r.identity || '._identity_' || a.attname, 'identity-column',
           jsonb_build_object('column', a.attname, 'generation', a.attidentity),
           jsonb_build_array(r.identity || '.' || a.attname)
    FROM relations r JOIN pg_catalog.pg_attribute a ON a.attrelid = r.oid
    WHERE a.attnum > 0 AND NOT a.attisdropped AND a.attidentity <> ''

    UNION ALL
    SELECT n.nspname || '.' || t.typname, 'enum',
           jsonb_build_object('name', t.typname, 'values', (
               SELECT jsonb_agg(e.enumlabel ORDER BY e.enumsortorder)
               FROM pg_catalog.pg_enum e WHERE e.enumtypid = t.oid
           ), 'variants', (
               SELECT jsonb_agg(e.enumlabel ORDER BY e.enumsortorder)
               FROM pg_catalog.pg_enum e WHERE e.enumtypid = t.oid
           )), jsonb_build_array(n.nspname)
    FROM pg_catalog.pg_type t JOIN user_namespaces n ON n.oid = t.typnamespace
    WHERE t.typtype = 'e'

    UNION ALL
    SELECT n.nspname || '.' || t.typname, 'domain',
           jsonb_build_object('name', t.typname, 'base-database-type-name', pg_catalog.format_type(t.typbasetype, t.typtypmod),
                              'nullable', NOT t.typnotnull, 'default-expression', COALESCE(t.typdefault, ''),
                              'sifr_type', 'str'), jsonb_build_array(n.nspname)
    FROM pg_catalog.pg_type t JOIN user_namespaces n ON n.oid = t.typnamespace
    WHERE t.typtype = 'd'

    UNION ALL
    SELECT n.nspname || '.' || t.typname, 'composite',
           jsonb_build_object('name', t.typname, 'attributes', COALESCE((
               SELECT jsonb_agg(jsonb_build_object('name', a.attname, 'database-type-name', pg_catalog.format_type(a.atttypid, a.atttypmod)) ORDER BY a.attnum)
               FROM pg_catalog.pg_attribute a WHERE a.attrelid = t.typrelid AND a.attnum > 0 AND NOT a.attisdropped
           ), '[]'::jsonb), 'fields', COALESCE((
               SELECT jsonb_object_agg(a.attname, 'str')
               FROM pg_catalog.pg_attribute a WHERE a.attrelid = t.typrelid AND a.attnum > 0 AND NOT a.attisdropped
           ), '{}'::jsonb)), jsonb_build_array(n.nspname)
    FROM pg_catalog.pg_type t
    JOIN user_namespaces n ON n.oid = t.typnamespace
    JOIN pg_catalog.pg_class c ON c.oid = t.typrelid AND c.relkind = 'c'
    WHERE t.typtype = 'c'

    UNION ALL
    SELECT n.nspname || '.' || t.typname, 'range',
           jsonb_build_object('name', t.typname, 'subtype-database-type-name', pg_catalog.format_type(r.rngsubtype, NULL),
                              'collation', CASE WHEN r.rngcollation = 0 THEN '' ELSE r.rngcollation::regcollation::text END),
           jsonb_build_array(n.nspname)
    FROM pg_catalog.pg_range r
    JOIN pg_catalog.pg_type t ON t.oid = r.rngtypid
    JOIN user_namespaces n ON n.oid = t.typnamespace

    UNION ALL
    SELECT n.nspname || '.' || t.typname, 'array',
           jsonb_build_object('name', t.typname, 'element-type-name', pg_catalog.format_type(t.typelem, NULL)),
           jsonb_build_array(n.nspname)
    FROM pg_catalog.pg_type t JOIN user_namespaces n ON n.oid = t.typnamespace
    WHERE t.typcategory = 'A' AND t.typelem <> 0

    UNION ALL
    SELECT n.nspname || '.' || p.proname || '._overload_' || md5(pg_catalog.pg_get_function_identity_arguments(p.oid)), 'function',
           jsonb_build_object('name', p.proname, 'identity-arguments', pg_catalog.pg_get_function_identity_arguments(p.oid),
                              'result-type-name', pg_catalog.pg_get_function_result(p.oid), 'kind', p.prokind,
                              'volatility', p.provolatile, 'strict', p.proisstrict,
                              'security-definer', p.prosecdef, 'definition', pg_catalog.pg_get_functiondef(p.oid),
                              'overload_namespace', n.nspname, 'overload_name', p.proname),
           jsonb_build_array(n.nspname)
    FROM pg_catalog.pg_proc p JOIN user_namespaces n ON n.oid = p.pronamespace

    UNION ALL
    SELECT n.nspname || '._operator._overload_' || md5(o.oprname || ':' || o.oprleft::text || ':' || o.oprright::text), 'operator',
           jsonb_build_object('name', o.oprname, 'left-type-name', o.oprleft::regtype::text,
                              'right-type-name', o.oprright::regtype::text, 'result-type-name', o.oprresult::regtype::text,
                              'overload_namespace', n.nspname, 'overload_name', o.oprname),
           jsonb_build_array(n.nspname)
    FROM pg_catalog.pg_operator o JOIN user_namespaces n ON n.oid = o.oprnamespace

    UNION ALL
    SELECT '_cast._overload_' || md5(c.castsource::text || ':' || c.casttarget::text), 'cast',
           jsonb_build_object('source-type-name', c.castsource::regtype::text, 'target-type-name', c.casttarget::regtype::text,
                              'context', c.castcontext, 'method', c.castmethod), '[]'::jsonb
    FROM pg_catalog.pg_cast c
    WHERE c.castsource IN (SELECT oid FROM pg_catalog.pg_type WHERE typnamespace IN (SELECT oid FROM user_namespaces))
       OR c.casttarget IN (SELECT oid FROM pg_catalog.pg_type WHERE typnamespace IN (SELECT oid FROM user_namespaces))

    UNION ALL
    SELECT n.nspname || '.' || c.collname, 'collation',
           jsonb_build_object('name', c.collname, 'provider', c.collprovider,
                              'deterministic', c.collisdeterministic,
                              'collate', COALESCE(c.collcollate, ''),
                              'ctype', COALESCE(c.collctype, '')), jsonb_build_array(n.nspname)
    FROM pg_catalog.pg_collation c JOIN user_namespaces n ON n.oid = c.collnamespace

    UNION ALL
    SELECT '_extension.' || e.extname, 'extension',
           jsonb_build_object('name', e.extname, 'version', e.extversion, 'relocatable', e.extrelocatable,
                              'namespace', n.nspname), '[]'::jsonb
    FROM pg_catalog.pg_extension e JOIN pg_catalog.pg_namespace n ON n.oid = e.extnamespace

    UNION ALL
    SELECT r.identity || '._trigger_' || t.tgname, 'trigger',
           jsonb_build_object('name', t.tgname, 'definition', pg_catalog.pg_get_triggerdef(t.oid, true),
                              'enabled', t.tgenabled), jsonb_build_array(r.identity)
    FROM pg_catalog.pg_trigger t JOIN relations r ON r.oid = t.tgrelid
    WHERE NOT t.tgisinternal

    UNION ALL
    SELECT '_server.capabilities', 'server-capability',
           jsonb_build_object('server-version-num', current_setting('server_version_num'),
                              'integer-datetimes', current_setting('integer_datetimes'),
                              'standard-conforming-strings', current_setting('standard_conforming_strings')),
           '[]'::jsonb

    UNION ALL
    SELECT '_dialect.postgresql', 'dialect-metadata',
           jsonb_build_object('server-version', current_setting('server_version'),
                              'server-encoding', current_setting('server_encoding'),
                              'default-collation', (
                                  SELECT d.datcollate FROM pg_catalog.pg_database d
                                  WHERE d.datname = pg_catalog.current_database()
                              )),
           '[]'::jsonb
)
SELECT identity, object_kind, semantic::text AS semantic_json, dependencies::text AS dependencies_json
FROM catalog_objects
ORDER BY identity, object_kind
