
create table USERS (
  ID       BIGSERIAL primary key,
  NAME     VARCHAR(256) unique,
  PASSWORD VARCHAR(256) not null
);


create table USER_ROLES (
  USER_ID         BIGSERIAL primary key,
  READ_ROLE       BOOL, --  default false,
  WRITE_ROLE      BOOL, --  default false,
  USER_ROLE       BOOL, --  default false, -- ??
  SUPER_USER_ROLE BOOL, --  default false,
  ADMIN_ROLE      BOOL, --  default false,

  constraint fk_user
    foreign key (USER_ID)
      references USERS(ID)
);