pub mod arknet;
pub mod peer;
// pub mod replicator; // TODO: Implement later
// pub mod client; // TODO: Implement later
// pub mod server; // TODO: Implement later

pub use arknet::*;
pub use peer::*;
// pub use replicator::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
