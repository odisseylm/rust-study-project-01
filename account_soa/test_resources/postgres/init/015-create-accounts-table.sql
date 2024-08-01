

create table ACCOUNTS
(
    -- Does it really make sense??
    -- internal ID (it can allow to change natural key format (IBAN format)
    -- if it is needed by new country laws)
    ID         UUID         not null primary key,
    -- natural key
    IBAN       IBAN         collate ENGLISH_CI not null unique,
    CLIENT_ID  CLIENT_ID    not null,
    -- type modifier is not allowed for type "citext"
    -- NAME       CITEXT(256)  not null,
    NAME       VARCHAR(256) not null,
    AMOUNT     AMOUNT       not null default 0 check (AMOUNT >= 0),
    CUR        CURRENCY     not null,
    CREATED_AT TIMESTAMPTZ  not null default CURRENT_TIMESTAMP,
    UPDATED_AT TIMESTAMPTZ  not null default CURRENT_TIMESTAMP,

    constraint FK_CLIENT_ID foreign key(CLIENT_ID) references CLIENTS(CLIENT_ID),
    constraint C_ACCOUNT_NAME unique (CLIENT_ID, NAME)
    -- check (length(name) >= 1 and length(name) <= 300),
);


-- create unique index USERS_UNIQUE_IDX on USERS(EMAIL) where DELETED = false

-- create table IP_RANGES
-- (
--     IP_RANGE IP4R,
--     exclude using gist (IP_RANGE with &&)
-- );
