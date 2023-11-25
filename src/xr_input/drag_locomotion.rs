use std::f32::consts::PI;

use bevy::{
    prelude::*,
    time::{Time, Timer, TimerMode},
};

use crate::{
    input::XrInput,
    resources::{XrFrameState, XrInstance, XrSession, XrViews},
};

use super::{
    actions::XrActionSets, oculus_touch::OculusController, trackers::OpenXRTrackingRoot, Hand,
    QuatConv, Vec3Conv,
};

#[derive(Resource)]
pub struct DragLocomotionConfig {
    pub allow_rotation: bool,
    pub allow_scale: bool,
    pub allow_up_down: bool
}

impl Default for DragLocomotionConfig {
    fn default() -> Self {
        Self {
            allow_rotation: true,
            allow_scale: true,
            allow_up_down: true
        }
    }
}

#[derive(Default)]
pub struct DragLocomotionState {
    drag_last: Option<(Hand, Vec3)>,
    _rotation_start: Option<(Vec3, Vec3)>,
    _scale_start: Option<f32>,
}

pub fn drag_locomotion(
    _time: Res<Time>,
    mut tracking_root_query: Query<(&mut Transform, With<OpenXRTrackingRoot>)>,
    oculus_controller: Res<OculusController>,
    frame_state: Res<XrFrameState>,
    xr_input: Res<XrInput>,
    _instance: Res<XrInstance>,
    session: Res<XrSession>,
    _views: ResMut<XrViews>,
    mut _gizmos: Gizmos,
    config_option: Option<ResMut<DragLocomotionConfig>>,
    mut drag_state: Local<DragLocomotionState>,
    action_sets: Res<XrActionSets>,
) {
    match config_option {
        Some(_) => (),
        None => {
            info!("no locomotion config");
            return;
        }
    }
    //i hate this but im too tired to think
    let mut config = config_option.unwrap();
    //lock frame
    let frame_state = *frame_state.lock().unwrap();
    //get controller
    let controller = oculus_controller.get_ref(&session, &frame_state, &xr_input, &action_sets);
    let root = tracking_root_query.get_single_mut();
    match root {
        Ok(mut position) => {
            //get the stick input and do some maths
            let left_squeezed = controller.squeeze(Hand::Left) > 0.5f32;
            let right_squeezed = controller.squeeze(Hand::Right) > 0.5f32;
            let both_squeezed = left_squeezed && right_squeezed;

            let should_traslate = left_squeezed || right_squeezed;
            let _should_rotate = both_squeezed && config.allow_rotation;
            let _should_scale = both_squeezed && config.allow_scale;

            if should_traslate {
                let dragging_hand = match right_squeezed {
                    true => Hand::Right,
                    false => Hand::Left,
                };
                let drag_position = controller.grip_space(dragging_hand).0.pose.position.to_vec3();
                
                let (hand, delta) = match drag_state.drag_last {
                    Some((hand, start)) if hand == dragging_hand => {
                        (hand, drag_position - start)
                    }
                    _ => {
                        (dragging_hand, Vec3::ZERO)
                    }
                };

                drag_state.drag_last = Some((hand, drag_position));
                position.0.translation += delta;
            }
        }
        Err(_) => info!("too many tracking roots"),
    }
}
