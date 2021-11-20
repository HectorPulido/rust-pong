use bevy::prelude::*;
use heron::prelude::*;
use rand::prelude::*;
use wasm_bindgen::prelude::*;

// SETUP STUFF
const RACKET_HEIGHT: f32 = 35.0;
const SCREEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 480.0;

fn spawn_camera(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle = asset_server.load("DottedLine.png");
    commands.spawn_bundle(SpriteBundle {
        material: materials.add(texture_handle.into()),
        transform: Transform {
            scale: Vec3::new(10.0, 10.0, 10.0),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn spawn_ball(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let texture_handle = asset_server.load("Ball.png");
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_handle.into()),
            transform: Transform {
                scale: Vec3::new(10.0, 10.0, 10.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Ball::new())
        .insert(CollisionShape::Sphere { radius: 10.0 })
        .insert(RigidBody::Dynamic)
        .insert(Velocity {
            linear: Ball::get_initial_speed(),
            ..Default::default()
        });
}

fn spawn_rackets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let texture_handle = asset_server.load("Ball.png");
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_handle.clone().into()),
            transform: Transform {
                translation: Vec3::new(-250.0, 0.0, 0.0),
                scale: Vec3::new(10.0, 50.0, 10.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Racket {
            is_player: true,
            ai_last_ball_ypos: 0.0,
        })
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(8.0, RACKET_HEIGHT, 0.0),
            border_radius: None,
        })
        .insert(RigidBody::KinematicVelocityBased)
        .insert(Velocity::default());

    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_handle.clone().into()),
            transform: Transform {
                translation: Vec3::new(250.0, 0.0, 0.0),
                scale: Vec3::new(10.0, 50.0, 10.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Racket {
            is_player: false,
            ai_last_ball_ypos: 0.0,
        })
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(8.0, RACKET_HEIGHT, 0.0),
            border_radius: None,
        })
        .insert(RigidBody::KinematicVelocityBased)
        .insert(Velocity::default());
}

fn spawn_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_style = TextStyle {
        font: asset_server.load("Minecraft.ttf"),
        font_size: 40.0,
        color: Color::WHITE,
    };

    let text = Text {
        sections: vec![
            TextSection {
                value: "0".to_string(),
                style: font_style.clone(),
            },
            TextSection {
                value: " - ".to_string(),
                style: font_style.clone(),
            },
            TextSection {
                value: "0".to_string(),
                style: font_style.clone(),
            },
        ],
        alignment: TextAlignment {
            vertical: VerticalAlign::Top,
            horizontal: HorizontalAlign::Center,
        },
    };

    commands.spawn_bundle(UiCameraBundle::default());

    commands
        .spawn_bundle(Text2dBundle {
            text: text,
            transform: Transform {
                translation: Vec3::new(0.0, 175.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ScoreText {});
}

// Components
struct ScoreText;

// Entity
struct Ball {
    left_score: i16,
    right_score: i16,
    hitting_dir: i8,
}
impl Ball {
    const BALL_INITIAL_SPEED: f32 = 400.0;

    fn new() -> Self {
        Ball {
            left_score: 0,
            right_score: 0,
            hitting_dir: 0,
        }
    }

    fn ball_reflect(
        &mut self,
        self_transform: &Transform,
        racket_transform: &Transform,
        velocity: &mut Velocity,
    ) {
        let ball_dir = velocity.linear.x.signum();

        // No refleja si es el mismo racket con el que choco anteriormente.
        if self.hitting_dir != 0 && self.hitting_dir == ball_dir as i8 {
            return;
        }

        let mut new_velocity = velocity.linear;

        new_velocity.y =
            (self_transform.translation.y - racket_transform.translation.y) / RACKET_HEIGHT;
        new_velocity.x = -new_velocity.x.signum();
        new_velocity = new_velocity.normalize() * Ball::BALL_INITIAL_SPEED;
        velocity.linear = new_velocity;

        self.hitting_dir = racket_transform.translation.x.signum() as i8;
    }

    fn get_initial_speed() -> Vec3 {
        let mut rng = rand::thread_rng();
        let direction: f32 = if rng.gen::<f32>() > 0.5 { 1.0 } else { -1.0 };

        // Que inicie con un angulo aleatorio y que nunca sea 0.
        // Esto para que al inicio la bola no se quede lineal
        Vec3::new(
            direction * Ball::BALL_INITIAL_SPEED,
            (rng.gen::<f32>() - 0.5).clamp(-0.1, 0.1) * 100.0,
            0.0,
        )
    }
}

struct Racket {
    is_player: bool,
    ai_last_ball_ypos: f32,
}

impl Racket {
    const PLAYER_SPEED: f32 = 200.0;
    const AI_SPEED: f32 = 200.0; // Debemos ser justos

    fn racket_ai(
        &mut self,
        self_transform: &Transform,
        ball_transform: &Transform,
        velocity: &mut Velocity,

        ball_velocity: &Velocity,
    ) {
        let diff = ball_transform.translation.y - self_transform.translation.y;
        let distance = diff.abs();

        // Vamos a hacerlo un poco mas natural
        let mut distance_to_move = 18.0;

        // Verificamos si la bola va hacia nuestro player
        // Si es asi entonces aumentamos la distancia necesaria para moverse
        if ball_velocity.linear.x < 0.0 {
            distance_to_move = 30.0;
        }

        // Si nos salimos de la distancia adecuada entonces guardamos el centro de la bola
        if distance >= distance_to_move {
            self.ai_last_ball_ypos = ball_transform.translation.y;
        }

        // Ahora Hacemos el movimiento pero con el centro guardado

        let diff = self.ai_last_ball_ypos - self_transform.translation.y;
        let distance = diff.abs();

        // Si no esta en una distancia adecuada para moverse entonces: UY KIETO
        velocity.linear.y = 0.0;

        // Una pequeña distancia para que no tiemble del frio xd
        if distance > 3.0 {
            velocity.linear.y = diff.clamp(-1.0, 1.0) * Racket::AI_SPEED;
        }
    }

    fn player_racket(keyboard_input: &Res<Input<KeyCode>>, velocity: &mut Velocity) {
        velocity.linear.y = if keyboard_input.pressed(KeyCode::Up) {
            Racket::PLAYER_SPEED
        } else if keyboard_input.pressed(KeyCode::Down) {
            -Racket::PLAYER_SPEED
        } else {
            0.0
        }
    }
}

// Systems
fn ball(
    mut query: Query<(&mut Ball, &mut Transform, &mut Velocity)>,
    mut text_query: Query<(&ScoreText, &mut Text)>,
) {
    let (_, mut score_text) = text_query.iter_mut().nth(0).unwrap();

    for (mut ball, mut transform, mut velocity) in query.iter_mut() {
        velocity.angular = AxisAngle::new(Vec3::new(0.0, 0.0, 1.0), 0.0);
        if transform.translation.y > 200.0 || transform.translation.y < -200.0 {
            velocity.linear.y *= -1.0;
        }

        if transform.translation.x > 310.0 || transform.translation.x < -310.0 {
            if transform.translation.x.signum() > 0.0 {
                ball.left_score += 1;
                score_text.sections[0].value = ball.left_score.to_string();
            } else {
                ball.right_score += 1;
                score_text.sections[2].value = ball.right_score.to_string();
            };

            velocity.linear = Ball::get_initial_speed();
            transform.translation = Vec3::new(0.0, 0.0, 0.0);

            ball.hitting_dir = 0;
        }
    }
}

fn racket(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Racket, &mut Transform, &mut Velocity), Without<Ball>>,
    ball: Query<(&Ball, &Transform, &Velocity), Without<Racket>>,
) {
    // Obtenemos la velocidad de la bola, para saber a que lado se mueve
    let (_, ball_transform, ball_velocity) = ball.iter().nth(0).unwrap();

    for (mut racket, transform, mut velocity) in query.iter_mut() {
        if racket.is_player {
            Racket::player_racket(&keyboard_input, &mut velocity);
            // Que no ande viajando libremente por el espacio, que le pasa.
            if (velocity.linear.y > 0.0
                && transform.translation.y > SCREEN_HEIGHT / 2.0 - RACKET_HEIGHT)
                || (velocity.linear.y < 0.0
                    && transform.translation.y < -SCREEN_HEIGHT / 2.0 + RACKET_HEIGHT)
            {
                velocity.linear.y = 0.0;
            }
        } else {
            racket.racket_ai(&transform, &ball_transform, &mut velocity, ball_velocity);
        }
    }
}

fn ball_collision(
    mut events: EventReader<CollisionEvent>,
    mut ball: Query<(&mut Ball, &Transform, &mut Velocity), Without<Racket>>,
    rackets: Query<(Entity, &Racket, &Transform), Without<Ball>>,
) {
    let (mut ball_component, ball_transform, mut ball_velocity) = ball.iter_mut().nth(0).unwrap();
    for event in events.iter() {
        match event {
            CollisionEvent::Started(d1, d2) => {
                let (_, _, racket_transform) = rackets
                    .iter()
                    .filter(|(entity, _, _)| {
                        [d1.rigid_body_entity(), d2.rigid_body_entity()].contains(entity)
                    })
                    .nth(0)
                    .unwrap();

                ball_component.ball_reflect(ball_transform, racket_transform, &mut ball_velocity);
            }
            CollisionEvent::Stopped(_, _) => {}
        }
    }
}

#[wasm_bindgen]
pub fn run() {
    let mut app = App::build();

    // Standalone
    #[cfg(not(target_arch = "wasm32"))]
    app.insert_resource(WindowDescriptor {
        title: "Pong".to_string(),
        width: SCREEN_WIDTH,
        height: SCREEN_HEIGHT,
        vsync: true,
        ..Default::default()
    });

    app.insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .add_startup_system(spawn_camera.system())
        .add_startup_system(spawn_ball.system())
        .add_startup_system(spawn_rackets.system())
        .add_startup_system(spawn_text.system())
        .add_system(ball.system())
        .add_system(racket.system())
        .add_system(ball_collision.system());

    // when building for Web, use WebGL2 rendering
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);

    app.run();
}
