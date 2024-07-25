# leptrino_ros2

Node for using Leptrino Force Torque sensor in ROS2

## usage

First, configure udev rule according to [leptrino-force-torque-sensor](https://github.com/Amelia10007/leptrino-force-torque-sensor-rs?tab=readme-ov-file)

To run in cargo,

```sh
cargo run
```

To run in ROS2, build the package

```sh
colcon build --symlink-install --packages-select leptrino_ros2
. install/setup.bash
```

and run the node

```sh
ros2 run leptrino_ros2 leptrino_ros2
```

## parameters

## dependencies

- futures
- leptrino-force-torque-sensor
- r2r
- tracing
- tracing-subscriber
