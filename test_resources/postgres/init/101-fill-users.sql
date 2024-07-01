
insert into USERS (ID, NAME, PASSWORD) values (1, 'client1', 'psw1');
insert into USERS (ID, NAME, PASSWORD) values (2, 'client2', 'psw2');
insert into USERS (ID, NAME, PASSWORD) values (3, 'client3', 'psw3');
insert into USERS (ID, NAME, PASSWORD) values (4, 'client4', 'psw4');

insert into USERS (ID, NAME, PASSWORD) values (21, 'vovan', 'qwerty');
insert into USERS (ID, NAME, PASSWORD) values (22, 'vovan-read', 'qwerty');
insert into USERS (ID, NAME, PASSWORD) values (23, 'vovan-write', 'qwerty');
insert into USERS (ID, NAME, PASSWORD) values (24, 'vovan-read-write', 'qwerty');
insert into USERS (ID, NAME, PASSWORD) values (25, 'vovan-user', 'qwerty');
insert into USERS (ID, NAME, PASSWORD) values (26, 'vovan-super-user', 'qwerty');
insert into USERS (ID, NAME, PASSWORD) values (27, 'vovan-admin', 'qwerty');


insert into USER_ROLES (USER_ID, READ_ROLE)  values (22, 'y');
insert into USER_ROLES (USER_ID, WRITE_ROLE) values (23, 'y');
insert into USER_ROLES (USER_ID, READ_ROLE, WRITE_ROLE) values (24, 'y', 'y');
--insert into USER_ROLES (USER_ID, USER_ROLE) values (25, 'y');
insert into USER_ROLES (USER_ID, SUPER_USER_ROLE) values (26, 'y');
insert into USER_ROLES (USER_ID, ADMIN_ROLE) values (27, 'y');
