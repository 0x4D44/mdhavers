//! The browser build (Emscripten) has no sockets or OpenSSL: each network entry of
//! `net.rs` answers with the runtime's usual error instead.

use super::*;

const ERR_NO_NET: &str = "Och! Thon network wark (DNS, TLS, SRTP, DTLS) isnae available in the browser";

macro_rules! no_net {
    ($($name:ident($($arg:ident),*);)*) => {$(
        #[export_name = concat!(stringify!($name), "__rsabi")]
        pub extern "C" fn $name($(_: rsabi_arg!($arg)),*) -> MdhRsResult {
            unsafe { mdh_finish(mdh_err(ERR_NO_NET)) }
        }
    )*};
}

macro_rules! rsabi_arg {
    ($arg:ident) => {
        MdhValue
    };
}

no_net! {
    __mdh_rs_dns_srv(service, domain);
    __mdh_rs_dns_naptr(domain);
    __mdh_rs_tls_client_new(config);
    __mdh_rs_tls_connect(tls, sock);
    __mdh_rs_tls_send(tls, buf);
    __mdh_rs_tls_recv(tls, max_len);
    __mdh_rs_tls_close(tls);
    __mdh_rs_srtp_create(config);
    __mdh_rs_srtp_protect(ctx, packet);
    __mdh_rs_srtp_unprotect(ctx, packet);
    __mdh_rs_dtls_server_new(config);
    __mdh_rs_dtls_handshake(dtls, sock);
}
