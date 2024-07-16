
create table CLIENT (
    USER_ID    BIGINT,   -- TODO: temporary
    CLIENT_ID  CLIENT_ID    not null,
    -- Should it be unique? Ot is it allowed to one physical person have several client ids&&
    EMAIL      EMAIL        collate ENGLISH_CI not null unique,
    -- main phone
    -- TODO: use strict format to avoid duplicates due to different format
    PHONE      PHONE        collate ENGLISH_CI not null unique,
);


create table ACCOUNTS
(
    -- Does it really make sense??
    -- internal ID (it can allow to change natural key format (IBAN format)
    -- if it is needed by new country laws)
    ID         UUID         not null primary key,
    -- natural key
    IBAN       IBAN         not null primary key,
    CLIENT_ID  CLIENT_ID    not null,
    NAME       CITEXT(256) not null,
    AMOUNT     AMOUNT       not null default 0 check (AMOUNT >= 0),
    CUR        CURRENCY     not null,
    CREATED_AT TIMESTAMP    not null default CURRENT_TIMESTAMP,
    UPDATED_AT TIMESTAMP    not null default CURRENT_TIMESTAMP,

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
