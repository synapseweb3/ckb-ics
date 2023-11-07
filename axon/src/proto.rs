pub mod connection {
    include!(concat!(env!("OUT_DIR"), "/connection.rs"));
}

pub mod channel {
    include!(concat!(env!("OUT_DIR"), "/channel.rs"));
}

pub mod commitment {
    include!(concat!(env!("OUT_DIR"), "/commitment.rs"));
}

pub mod client {
    include!(concat!(env!("OUT_DIR"), "/client.rs"));
}
