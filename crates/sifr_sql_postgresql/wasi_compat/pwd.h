#ifndef SIFR_WASI_PWD_H
#define SIFR_WASI_PWD_H

/*
 * libpg_query includes PostgreSQL's general port header, which includes
 * pwd.h. The selected parser translation units do not call the account APIs.
 * WASI intentionally has no process account database, so the compiler
 * component supplies only the declaration shape required by those headers.
 */
struct passwd {
    char *pw_name;
    char *pw_passwd;
    unsigned int pw_uid;
    unsigned int pw_gid;
    char *pw_gecos;
    char *pw_dir;
    char *pw_shell;
};

#endif
