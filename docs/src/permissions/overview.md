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
|id|integer|
|name|text|
|owner_name|text|
|owner_homeserver|text|
|admin_group|integer (nullable)|

### Counters
|Field|Data type|
|---|---|
|id|text|
|value|integer|
