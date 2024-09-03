

create table CLIENTS (
    CLIENT_ID  CLIENT_ID     primary key,
    -- Should it be unique? Ot is it allowed to one physical person have several client ids&&
    EMAIL      EMAIL         collate ENGLISH_CI not null unique,
    -- main phone
    -- TODO: use strict format to avoid duplicates due to different format
    PHONE         PHONE      collate ENGLISH_CI not null unique,
    BIRTHDAY      DATE       not null,
    ACTIVE        BOOL       not null default 'n',
    BUSINESS_USER BOOL       not null default 'n',
    SUPER_BUSINESS_USER BOOL not null default 'n'
);

-- It should have more restricted access
create table CLIENTS_CREDS (
    CLIENT_ID  CLIENT_ID    primary key, -- and foreign key
    PSW        VARCHAR(256) not null,
    PSW_HASH   VARCHAR(256) not null,

    constraint FK_CLIENT_ID foreign key(CLIENT_ID) references CLIENTS(CLIENT_ID)
);

-- Optional/extra fields, Use outer joins for this table.
create table CLIENTS_EXT_INFO (
    CLIENT_ID  CLIENT_ID     primary key, -- and foreign key
    ADDRESS_1      EMAIL     collate ENGLISH_CI not null unique,
    -- T O D O: use strict format to avoid duplicates due to different format
    PHONE_ALT_1    PHONE     collate ENGLISH_CI not null unique,
    -- T O D O: use strict format to avoid duplicates due to different format
    PHONE_ALT_2    PHONE     collate ENGLISH_CI not null unique,

    constraint FK_CLIENT_ID foreign key(CLIENT_ID) references CLIENTS(CLIENT_ID)
);
