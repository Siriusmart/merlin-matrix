use matrix_sdk::{
    Client,
    ruma::events::room::member::{MembershipChange, OriginalSyncRoomMemberEvent},
};
use tracing::*;

use crate::org::{Database, users::User};

/// In case of member joined room event, add it to the users table if not exist
#[instrument(skip_all)]
pub async fn on_maybe_new_user(event: OriginalSyncRoomMemberEvent, client: Client) {
    if event.sender == client.user_id().unwrap() {
        return;
    }

    if matches!(
        event.membership_change(),
        MembershipChange::Joined
            | MembershipChange::InvitationAccepted
            | MembershipChange::KnockAccepted
    ) {
        let res = User::ensure_created(
            &mut Database::conn(),
            event.sender.localpart().to_string(),
            event.sender.server_name().to_string(),
        );

        if let Err(err) = res {
            error!(
                "Could not create user for m_user_id={} reason={err:?}",
                event.sender
            )
        }
    }
}
