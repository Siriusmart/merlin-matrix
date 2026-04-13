# Permissions

The goal is a role based permission system similar to Discord.

Since Matrix does not have the concept of communities (disjoint set of channels), the permission context of each room needs to be set manually.

- A group is a collection of users.
- A group can be edited by the group owner (who does not have to be in the group) or another group.
- A context is owned by a group.
- Only users in the owner group of a context can edit the context permissions.
- Only users with permission 90+ in a room can switch context of the room.

### Groups
|Field|Data type|
|---|---|
|group_id|int|
|name|text|
|owner_id|int|
|admin_group_id|int (nullable)|

### User
|Field|Data type|
|---|---|
|user_id|int|
|name|text|
|homeserver|text|

### Context
|Field|Data type|
|---|---|
|context_id|integer|
|name|text|
|description|text|
|owner_id|int|
|admin_group|integer (nullable)|

### Permission
|Field|Data type|
|---|---|
|permission_id|int|
|qualifier|text|

### Room
|Field|Data type|
|---|---|
|room_id|integer|
|matrix_room_id|integer|
|matrix_room_homeserver|text|
|context_id|int (nullable)|

### ContextPermission
|Field|Data type|
|---|---|
|context_id|int|
|permission_id|int|
|priority|int|
|allowed|bool|
