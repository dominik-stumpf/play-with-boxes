use bevy::prelude::*;
use bevy_xpbd_2d::prelude::*;
use derive_more::{Add, Mul};
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};

use super::shared::color_from_id;
use lightyear::client::components::LerpFn;
use lightyear::prelude::*;
use lightyear::utils::bevy_xpbd_2d::*;

pub const BALL_SIZE: f32 = 15.0;
pub const PLAYER_SIZE: f32 = 40.0;

// For prediction, we want everything entity that is predicted to be part of the same replication group
// This will make sure that they will be replicated in the same message and that all the entities in the group
// will always be consistent (= on the same tick)
pub const REPLICATION_GROUP: ReplicationGroup = ReplicationGroup::new_id(1);

// Player
#[derive(Bundle)]
pub struct PlayerBundle {
    id: PlayerId,
    position: Position,
    color: ColorComponent,
    replicate: Replicate,
    physics: PhysicsBundle,
    inputs: InputManagerBundle<PlayerActions>,
    // IMPORTANT: this lets the server know that the entity is pre-predicted
    // when the server replicates this entity; we will get a Confirmed entity which will use this entity
    // as the Predicted version
    pre_predicted: PrePredicted,
}

impl PlayerBundle {
    pub fn new(id: ClientId, position: Vec2, input_map: InputMap<PlayerActions>) -> Self {
        let color = color_from_id(id);
        Self {
            id: PlayerId(id),
            position: Position(position),
            color: ColorComponent(color),
            replicate: Replicate {
                // NOTE (important): all entities that are being predicted need to be part of the same replication-group
                //  so that all their updates are sent as a single message and are consistent (on the same tick)
                replication_group: REPLICATION_GROUP,
                // TODO: improve this! this should depend on the predict_all settings
                // We still need to specify the interpolation/prediction target for this local entity
                // in the case where we're running in HostServer mode
                prediction_target: NetworkTarget::All,
                ..default()
            },
            physics: PhysicsBundle::player(),
            inputs: InputManagerBundle::<PlayerActions> {
                action_state: ActionState::default(),
                input_map,
            },
            pre_predicted: PrePredicted::default(),
        }
    }
}

// Ball
#[derive(Bundle)]
pub struct BallBundle {
    position: Position,
    color: ColorComponent,
    replicate: Replicate,
    marker: BallMarker,
    physics: PhysicsBundle,
}

impl BallBundle {
    pub fn new(position: Vec2, color: Color, predicted: bool) -> Self {
        let mut replicate = Replicate {
            replication_target: NetworkTarget::All,
            ..default()
        };
        if predicted {
            replicate.prediction_target = NetworkTarget::All;
            replicate.replication_group = REPLICATION_GROUP;
        } else {
            replicate.interpolation_target = NetworkTarget::All;
        }
        Self {
            position: Position(position),
            color: ColorComponent(color),
            replicate,
            physics: PhysicsBundle::ball(),
            marker: BallMarker,
        }
    }
}

#[derive(Bundle)]
pub struct PhysicsBundle {
    pub collider: Collider,
    pub collider_density: ColliderDensity,
    pub rigid_body: RigidBody,
}

impl PhysicsBundle {
    pub fn ball() -> Self {
        Self {
            collider: Collider::circle(BALL_SIZE),
            collider_density: ColliderDensity(0.05),
            rigid_body: RigidBody::Dynamic,
        }
    }

    pub fn player() -> Self {
        Self {
            collider: Collider::rectangle(PLAYER_SIZE, PLAYER_SIZE),
            collider_density: ColliderDensity(0.2),
            rigid_body: RigidBody::Dynamic,
        }
    }
}

// Components
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct PlayerId(pub ClientId);

#[derive(Component, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ColorComponent(pub Color);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BallMarker;

#[component_protocol(protocol = "MyProtocol")]
pub enum Components {
    #[protocol(sync(mode = "once"))]
    PlayerId(PlayerId),
    #[protocol(sync(mode = "once"))]
    ColorComponent(ColorComponent),
    #[protocol(sync(mode = "once"))]
    BallMarker(BallMarker),
    // You need to specify how to do interpolation for the component
    // Normally LinearInterpolation is fine, but it's not possible for xpbd's components
    // as they do not implement Mul<f32> and Add<Self>
    // Instead, lightyear already implemented the interpolation for xpbd's components (although you could also implement it yourself)
    //
    // Then you can also specify how to correct the component when there is a mispredictions
    // The default is `InstantCorrector` which just snaps to the corrected value
    // You can also use `InterpolatedCorrector` which will re-use your interpolation function to
    // interpolate smoothly from the previously predicted value to the newly corrected value
    #[protocol(sync(
        mode = "full",
        lerp = "PositionLinearInterpolation",
        corrector = "InterpolatedCorrector"
    ))]
    Position(Position),
    #[protocol(sync(
        mode = "full",
        lerp = "RotationLinearInterpolation",
        corrector = "InterpolatedCorrector"
    ))]
    Rotation(Rotation),
    // NOTE: correction is only needed for components that are visually displayed!
    #[protocol(sync(mode = "full", lerp = "LinearVelocityLinearInterpolation"))]
    LinearVelocity(LinearVelocity),
    #[protocol(sync(mode = "full", lerp = "AngularVelocityLinearInterpolation"))]
    AngularVelocity(AngularVelocity),
}

// Channels

#[derive(Channel)]
pub struct Channel1;

// Messages

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Message1(pub usize);

#[message_protocol(protocol = "MyProtocol")]
pub enum Messages {
    Message1(Message1),
}

// Inputs

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum PlayerActions {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum AdminActions {
    SendMessage,
    Reset,
}

impl LeafwingUserAction for PlayerActions {}
impl LeafwingUserAction for AdminActions {}

// Protocol

protocolize! {
    Self = MyProtocol,
    Message = Messages,
    Component = Components,
    Input = (),
    LeafwingInput1 = PlayerActions,
    LeafwingInput2 = AdminActions,
}

pub fn protocol() -> MyProtocol {
    let mut protocol = MyProtocol::default();
    protocol.add_channel::<Channel1>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
        ..default()
    });
    protocol
}
