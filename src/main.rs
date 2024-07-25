use leptrino_force_torque_sensor::serialport;
use leptrino_force_torque_sensor::{LeptrinoSensor, Product};
use tracing::{error, info, Level};
use tracing_subscriber;

fn search_usb_sensor_path() -> Result<Option<String>, serialport::Error> {
    // Leptrino vendor ID.
    let vendor_id = 0x0483;
    // NOTE: The following product-ID may be specific for PFS055YA251U6.
    let product_id = 0x5740;

    let ports = serialport::available_ports()?;
    let path = ports
        .into_iter()
        .filter(move |port| match &port.port_type {
            // Takes only USB-connected device
            serialport::SerialPortType::UsbPort(usb) => {
                usb.vid == vendor_id && usb.pid == product_id
            }
            _ => false,
        })
        .map(|sensor_port| sensor_port.port_name)
        .next();

    Ok(path)
}

fn initialize_sensor() -> Option<LeptrinoSensor> {
    let path = match search_usb_sensor_path() {
        Ok(Some(path)) => path,
        Ok(None) => {
            println!("No leptrino sensor is connected.");
            return None;
        }
        Err(e) => {
            println!("{}", e);
            return None;
        }
    };
    info!("leptrino sensor path: {:?}", path);
    info!("startting sensor connection...");
    let product_kind = Product::Pfs055Ya251U6;
    let mut sensor = match LeptrinoSensor::open(product_kind, path) {
        Ok(sensor) => sensor,
        Err(e) => {
            println!("{}", e);
            return None;
        }
    };

    info!("sensor connection established.");

    // set zero
    std::thread::sleep(sensor.inner_port().timeout());
    match sensor.update() {
        Ok(w) => info!("zero-point: {:?}", w),
        Err(e) => info!("{}", e),
    }
    sensor.zeroed();

    info!("zero-point set.");

    // return sensor
    Some(sensor)
}

fn main() {
    // Initialize logger
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Initialize sensor
    let mut sensor: LeptrinoSensor = initialize_sensor().expect("Sensor initialization failed.");
    info!("Sensor initialized");

    info!("start reading sensor data...");

    // initialize ros2 node
    let ctx = r2r::Context::create().expect("Failed to create r2r context.");
    let mut node =
        r2r::Node::create(ctx, "leptrino_sensor", "").expect("Failed to create r2r node.");
    let pub_wrench = node
        .create_publisher::<r2r::geometry_msgs::msg::Wrench>("/wrench", r2r::QosProfile::default())
        .expect("Failed to create r2r publisher.");

    info!("ros2 node initialized.");
    info!("start publishing sensor data...");

    // publish sensor data
    loop {
        std::thread::sleep(sensor.inner_port().timeout());
        match sensor.update() {
            Ok(w) => {
                pub_wrench
                    .publish(&r2r::geometry_msgs::msg::Wrench {
                        force: r2r::geometry_msgs::msg::Vector3 {
                            x: w.force.x,
                            y: w.force.y,
                            z: w.force.z,
                        },
                        torque: r2r::geometry_msgs::msg::Vector3 {
                            x: w.torque.x,
                            y: w.torque.y,
                            z: w.torque.z,
                        },
                    })
                    .unwrap_or_else(|e| error!("Failed to publish: {}", e));
            }
            Err(e) => error!("{}", e),
        }
    }
}
