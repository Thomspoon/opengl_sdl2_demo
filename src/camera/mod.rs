use cgmath::{Angle, InnerSpace, Matrix4, Rad, Point3, Vector3};
use cgmath::EuclideanSpace;

const DEFAULT_YAW: f32 = -90.0;
const DEFAULT_PITCH: f32 = 0.0;
const DEFAULT_SPEED: f32 = 2.5;
const DEFAULT_SENSITIVITY: f32 = 0.1;
const DEFAULT_ZOOM: f32 = 45.0;

const ZOOM_LOW: f32 = 1.0;
const ZOOM_HIGH: f32 = 45.0;

pub enum CameraMovement {
    FORWARD,
    BACKWARD,
    LEFT,
    RIGHT,
}

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    // Camera Vectors
    position: Vector3<f32>,
    up: Vector3<f32>,
    direction: Vector3<f32>,
    world_up: Vector3<f32>,
    right: Vector3<f32>,

    // Euler angles
    yaw: f32,
    pitch: f32,

    // Camera options
    speed: f32,
    sensitivity: f32,
    zoom: f32,
}

impl Camera {
    pub fn new(position: Vector3<f32>, up: Vector3<f32>, pitch: f32, yaw: f32) -> Camera {
        let new_cam = Camera {
            position,
            world_up: up,
            up: Vector3::new(0.0, 0.0, 0.0),
            direction: Vector3::new(0.0, 0.0, 0.0),
            right: Vector3::new(0.0, 0.0, 0.0),
            yaw: yaw,
            pitch: pitch,
            speed: DEFAULT_SPEED,
            sensitivity: DEFAULT_SENSITIVITY,
            zoom: DEFAULT_ZOOM,
        };

        new_cam.recalculate_vectors();

        new_cam
    }

    pub fn get_view(self) -> Matrix4<f32> {
        Matrix4::look_at(Point3::from_vec(self.position), 
                         Point3::from_vec(self.position + self.direction), 
                         self.up)
    }

    pub fn get_zoom(self) -> f32 {
        self.zoom
    }

    pub fn move_position(mut self, move_type: CameraMovement, delta: f32) {
        let velocity = self.speed * delta;

        match move_type {
            CameraMovement::FORWARD => self.position += self.direction * velocity,
            CameraMovement::BACKWARD => self.position -= self.direction * velocity,
            CameraMovement::LEFT => self.position -= self.right * velocity,
            CameraMovement::RIGHT => self.position += self.right * velocity,
        }
    }

    pub fn move_rotation(mut self, yaw: f32, pitch: f32) {
        self.yaw += self.sensitivity * yaw;
        self.pitch += self.sensitivity * pitch;

        self.recalculate_vectors();
    }

    pub fn move_zoom(mut self, zoom: f32) {
        self.zoom = f32::min(zoom, ZOOM_HIGH);
        self.zoom = f32::max(zoom, ZOOM_LOW);
    }

    fn recalculate_vectors(mut self) {
        self.direction = Vector3::new(Rad(self.yaw).cos() * Rad(self.pitch).cos(), 
                                     Rad(self.pitch).sin(), 
                                     Rad(self.yaw).sin() * Rad(self.pitch).cos()).normalize();
        self.right = self.direction.cross(self.world_up).normalize();
        self.up    = self.right.cross(self.direction).normalize();
    }
}
