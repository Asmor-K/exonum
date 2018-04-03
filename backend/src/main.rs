extern crate advanced_cryptocurrency;
extern crate exonum;
extern crate exonum_configuration;

use advanced_cryptocurrency as cryptocurrency;
use exonum::helpers;
use exonum::helpers::fabric::NodeBuilder;
use exonum_configuration as configuration;

fn main() {
    exonum::crypto::init();
    helpers::init_logger().unwrap();

    let node = NodeBuilder::new()
        .with_service(Box::new(configuration::ServiceFactory))
        .with_service(Box::new(cryptocurrency::ServiceFactory));
    node.run();
}
