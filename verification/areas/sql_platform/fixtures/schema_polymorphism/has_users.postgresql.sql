CREATE TABLE public.users (
    id bigint PRIMARY KEY,
    email text NOT NULL UNIQUE
);
