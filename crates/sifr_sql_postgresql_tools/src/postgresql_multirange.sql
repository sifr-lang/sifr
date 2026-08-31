SELECT n.nspname || '.' || mt.typname AS identity,
       'range' AS object_kind,
       jsonb_build_object(
           'name', mt.typname,
           'subtype-database-type-name', pg_catalog.format_type(r.rngsubtype, NULL),
           'multirange', true,
           'collation', CASE WHEN r.rngcollation = 0 THEN '' ELSE r.rngcollation::regcollation::text END
       )::text AS semantic_json,
       jsonb_build_array(n.nspname || '.' || rt.typname)::text AS dependencies_json
FROM pg_catalog.pg_range r
JOIN pg_catalog.pg_type rt ON rt.oid = r.rngtypid
JOIN pg_catalog.pg_type mt ON mt.oid = r.rngmultitypid
JOIN pg_catalog.pg_namespace n ON n.oid = mt.typnamespace
WHERE r.rngmultitypid <> 0
  AND n.nspname <> 'information_schema'
  AND n.nspname NOT LIKE 'pg\_%' ESCAPE '\'
ORDER BY identity
