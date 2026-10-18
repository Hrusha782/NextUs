//! Names the app owns, which a `.nxt` program may not use.
//!
//! Account-level behaviour — logging out, changing a nickname, checking a
//! password — is fixed app behaviour living in the non-programmable chrome.
//! It is not part of the language, and a design or a build must not be able
//! to reach it.
//!
//! This matters because of who writes a program versus who runs it. In the
//! shell a program's author and its user are the same person. A **build** is
//! written by one chat owner and rendered by every member of the chat, so an
//! account action in a build acts on somebody else's account: a chat owner
//! could put up a convincing "confirm your password" panel, or log members
//! out. Rather than scope these per context, they are removed from the
//! language outright.
//!
//! Both surfaces are covered: the `action:` property, and the behaviour ids
//! the app binds by `data-nxt-id`.

/// Values of `action:` that the language does not accept.
///
/// These are account-level commands. The app performs them from its own
/// chrome; a `.nxt` program cannot request them.
pub const RESERVED_ACTIONS: &[&str] = &["change_nick", "logout", "verify_password"];

/// Element ids the app binds to account-level behaviour.
///
/// `checkpass` is the id form of `verify_password` and `nick` of
/// `change_nick`; `whoami` and `logout` are the rest of the session surface.
pub const RESERVED_IDS: &[&str] = &["checkpass", "logout", "nick", "whoami"];

/// Why a given name is refused, or `None` if the program may use it.
pub fn reserved_action_reason(action: &str) -> Option<&'static str> {
    RESERVED_ACTIONS.contains(&action).then_some(
        "it is an account-level action performed by the app itself, not by a design or a build",
    )
}

/// Why a given id is refused, or `None` if the program may use it.
pub fn reserved_id_reason(id: &str) -> Option<&'static str> {
    RESERVED_IDS.contains(&id).then_some(
        "it is bound to account-level behaviour in the app's own chrome, which a design or a build cannot drive",
    )
}
