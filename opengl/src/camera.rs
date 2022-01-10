extern crate nalgebra_glm as glm;
use glm::*;

extern crate glium;
use glium::glutin;

pub struct CameraState {
    pub mouse_sensitivity: f32,
    pub movement_speed: f32,

    pub position: glm::TVec3<f32>,
    pub rotation: glm::TVec2<f32>, //yaw, pitch

    view: glm::TMat4<f32>,

    moving_up: bool,
    moving_left: bool,
    moving_down: bool,
    moving_right: bool,
    moving_forward: bool,
    moving_backward: bool,
}

impl CameraState {
    pub fn new() -> CameraState {
        CameraState {
            mouse_sensitivity: 0.1,
            movement_speed: 10.0,
            rotation: glm::zero(),
            position: glm::zero(),
            view: glm::one(),
            moving_up: false,
            moving_left: false,
            moving_down: false,
            moving_right: false,
            moving_forward: false,
            moving_backward: false,
        }
    }

    fn calc_view_matrix(&self, orientation: &glm::Qua<f32>) -> glm::TMat4<f32>
	{
		let reverse_orient = orientation.conjugate();
		let rot = glm::quat_to_mat4(&reverse_orient);
        let translation = glm::translate(&glm::identity(), &(-self.position));

		glm::transpose(&(rot * translation))
	}

    pub fn front(&self) -> glm::TVec3<f32> {
        -glm::vec3(self.view[2], self.view[6], self.view[10])
    }

    pub fn right(&self) -> glm::TVec3<f32> {
        glm::vec3(self.view[0], self.view[4], self.view[8])
    }

    pub fn up(&self) -> glm::TVec3<f32> {
        glm::vec3(self.view[1], self.view[5], self.view[9])
    }

    pub fn get_view_matrix(&self) -> glm::TMat4<f32>
	{
		self.view
	}

    pub fn update_camera_vectors(&mut self, dt: f32)
	{
        let around_y = glm::quat_angle_axis(glm::radians(&glm::vec1(self.rotation.y)).x, &glm::vec3(1.0, 0.0, 0.0)); //yaw
        let around_x = glm::quat_angle_axis(glm::radians(&glm::vec1(self.rotation.x)).x, &glm::vec3(0.0, 1.0, 0.0)); //pitch

		let orientation = around_y * around_x;

        self.view = self.calc_view_matrix(&orientation);
        let front = self.front();
        let right = self.right();
		let up    = self.up();

        let mut m = vec3(0.0, 0.0, 0.0);

        if self.moving_up { m += up; }
        if self.moving_down { m -= up; }
        if self.moving_left { m -= right; }
        if self.moving_right { m += right; }
        if self.moving_forward { m += front; }
        if self.moving_backward { m -= front; }

        if glm::is_null(&m, 0.001) { return; }
        m = glm::normalize(&m);

        self.position += m*dt*self.movement_speed;
	}

    pub fn process_input_cursor(&mut self, cur: glm::TVec2<f32>) {
        self.rotation += cur * self.mouse_sensitivity;
        self.rotation = glm::modf_vec(&self.rotation, &glm::vec2(360.0, 360.0));
    }

    pub fn process_input_keyboard(&mut self, event: &glutin::event::WindowEvent<'_>) {
        let input = match *event {
            glutin::event::WindowEvent::KeyboardInput { input, .. } => input,
            _ => return,
        };
        let pressed = input.state == glutin::event::ElementState::Pressed;
        let key = match input.virtual_keycode {
            Some(key) => key,
            None => return,
        };
        match key {
            glutin::event::VirtualKeyCode::E => self.moving_up = pressed,
            glutin::event::VirtualKeyCode::Q => self.moving_down = pressed,
            glutin::event::VirtualKeyCode::A => self.moving_left = pressed,
            glutin::event::VirtualKeyCode::D => self.moving_right = pressed,
            glutin::event::VirtualKeyCode::W => self.moving_forward = pressed,
            glutin::event::VirtualKeyCode::S => self.moving_backward = pressed,
            _ => (),
        };
    }
}