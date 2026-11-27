//! `/_app`: what app serves that is not a page. With `/auth`, it is what
//! the gateway in front of imkitchen routes to app whoever asks: the same
//! address at the root of the main domain is www's for a browser that is
//! not signed in. The assets are served under it too
//! ([`assets`](crate::assets)).

pub mod health;

use topcoat::router::segment;

// A module whose name starts with `_` adds nothing to an address unless it
// says so.
segment!(kind: Static, rename: "_app");
