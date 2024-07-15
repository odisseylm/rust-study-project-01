
create table ACCOUNTS
(
    ID         BIGSERIAL primary key,
    NATURAL_KEY ACCOUNT_ID  not null unique,
    NAME       VARCHAR(256) not null,
    CLIENT_ID  BIGINT       not null,
    CREATED_AT TIMESTAMP    not null default CURRENT_TIMESTAMP,
    UPDATED_AT TIMESTAMP,
    AMOUNT     AMOUNT       not null default 0 check (AMOUNT >= 0),
    CURRENCY   CURRENCY     not null,

    constraint FK_CLIENT_ID foreign key(CLIENT_ID) references USERS(ID),
    constraint C_ACCOUNT_NAME unique (CLIENT_ID, NAME)
    -- check (length(name) >= 1 and length(name) <= 300),
);


-- create unique index USERS_UNIQUE_IDX on USERS(EMAIL) where DELETED = false

-- create table IP_RANGES
-- (
--     IP_RANGE IP4R,
--     exclude using gist (IP_RANGE with &&)
-- );
