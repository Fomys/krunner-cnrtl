// More advanced server example.

// This is supposed to look like a D-Bus service that allows the user to manipulate storage devices.

// Note: in the dbus-codegen/example directory, there is a version of this example where dbus-codegen
// was used to create some boilerplate code - feel free to compare the two examples.

use std::sync::Arc;

use dbus::{tree, Path};
use dbus::tree::{Interface, MTFn, Tree};
use dbus::ffidisp::Connection;
use std::collections::HashMap;
use dbus::arg::Variant;
use std::collections::hash_map::RandomState;
use dbus::arg::messageitem::MessageItem::Str;
use std::process::Command;

// Our storage device
#[derive(Debug)]
struct Device {
    description: String,
    path: Path<'static>,
}

// Every storage device has its own object path.
// We therefore create a link from the object path to the Device.
#[derive(Copy, Clone, Default, Debug)]
struct TData;
impl tree::DataType for TData {
    type Tree = ();
    type ObjectPath = Arc<Device>;
    type Property = ();
    type Interface = ();
    type Method = ();
    type Signal = ();
}


impl Device {
    // Creates a "test" device (not a real one, since this is an example).
    fn new() -> Device {
        Device {
            description: "A simple krunner test".to_string(),
            path: "/".into(),
        }
    }
}

// Here's where we implement the code for our interface.
#[allow(unused_must_use)]
fn create_iface() -> Interface<MTFn<TData>, TData> {
    let f = tree::Factory::new_fn();

    f.interface("org.kde.krunner1", ())
         // ...and so is the "description" property
         .add_m(f.method("Actions", (), move |m| {
             Ok(vec!(m.msg.method_return()))
         })
             .inarg::<Variant<()>,_>("msg")
             .outarg::<Vec<(String, String, String)>,_>("arg_1")
         )
        .add_m(
            f.method("Run", (), move |m| {
                let args = m.msg.get_items();
                if let Str(pattern) = &args[0] {
                    Command::new("sh")
                        .arg("-c")
                        .arg(format!("xdg-open https://www.cnrtl.fr/definition/{}", pattern))
                        .output();
                }
                Ok(vec!(m.msg.method_return()))
            })
                .inarg::<String,_>("matchId")
                .inarg::<String,_>("actionId")
        )
        .add_m(
            f.method("Match", (), move |m| {
                let args = m.msg.get_items();
                if let Str(pattern) = &args[0] {
                    if pattern.starts_with("def ") {
                        let word = pattern.trim_start_matches("def ");
                        let ret = m.msg.method_return();
                        let o: HashMap<&str, Variant<&str>, RandomState> = HashMap::new();
                        return Ok(vec!(ret.append1(vec!((word, format!("Definition: {}", word), "internet-web-browser", 100, 1.0, o)))));
                    }
                }


                let ret = m.msg.method_return();
                Ok(vec!(ret))
            })
                .inarg::<String,_>("query")
                .outarg::<Vec<(String, String, String, i32, f64, HashMap<String, Variant<()>>)>,_>("arg_1")
        )
}

fn create_tree(device: Arc<Device>, iface: &Arc<Interface<MTFn<TData>, TData>>)
               -> tree::Tree<MTFn<TData>, TData> {

    let f = tree::Factory::new_fn();
    let mut tree = f.tree(());
    tree = tree.add(f.object_path(device.path.clone(), device.clone())
            .introspectable()
            .add(iface.clone())
        );

    tree
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Create our bogus devices
    let device: Arc<Device> = Arc::new(Device::new());

    // Create tree
    let iface: Interface<MTFn<TData>, TData> = create_iface();
    let tree: Tree<MTFn<TData>, TData> = create_tree(device, &Arc::new(iface));

    // Setup DBus connectio
    let c: Connection = Connection::new_session()?;
    c.register_name("com.louischauvet.krunner_cnrtl", 0)?;
    tree.set_registered(&c, true)?;

    // ...and serve incoming requests.
    c.add_handler(tree);
    loop {
        // Wait for incoming messages. This will block up to one second.
        // Discard the result - relevant messages have already been handled.
        c.incoming(1000).next();

        // Do all other things we need to do in our main loop.
    }
}

fn main() {
    if let Err(e) = run() {
        println!("{}", e);
    }
}
