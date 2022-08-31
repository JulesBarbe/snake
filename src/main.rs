use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::time::FixedTimestep;
use rand::prelude::random;
use bevy_editor_pls::prelude::*;
use bevy::app::AppExit;

//TODO: make movement time step scale with tail size (cant scale fixed timestep >:( )

const WINDOW_HEIGHT: f32 = 1000.0;
const WINDOW_WIDTH: f32 = 1000.0;

const SNAKE_HEAD_COLOR: Color = Color::WHITE;
const SNAKE_SEGMENT_COLOR: Color = Color::GRAY;
const FOOD_COLOR: Color = Color::RED;

const SNAKE_SIZE: f32 = 0.8;
const SEGMENT_SIZE: f32 = 0.6;
const FOOD_SIZE: f32 = 0.4;

const MOVEMENT_TIME_STEP: f64 = 0.2;  // move every x seconds
// const FOOD_EATEN_STEP: f64 = 0.005;
const STEP_SIZE: i32 = 1;

const ARENA_WIDTH: u32 = 15;
const ARENA_HEIGHT: u32 = 15;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(SystemLabel)]
enum MySystems {
    LAYOUT
}

fn main() {
    App::new()
    
    // RESOURCES
    .insert_resource(WindowDescriptor {
        title: "Snake lol".to_string(),
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
        ..default()
    })
    .insert_resource(ClearColor(Color::BLACK))
    .insert_resource(SnakeSegments::default())
    .insert_resource(LastTailPosition::default())
    .insert_resource(LastUserInput::default())
    .insert_resource(NumEaten::default())

    // SETUP
    .add_startup_system(spawn_snake)
    .add_startup_system(setup_camera)

    // INPUT
    .add_system(key_input.before(snake_movement))
    .add_event::<KeyboardInput>()
    .add_system(bevy::window::close_on_esc)

    // MOVEMENT
    .add_system_set(
        SystemSet::new()
            .with_run_criteria(FixedTimestep::step(MOVEMENT_TIME_STEP))
            .with_system(snake_movement)
            .with_system(snake_eating.after(snake_movement))
            .with_system(snake_growth.after(snake_eating))
    )

    // GAME OVER
    .add_system(game_over.after(snake_movement))

    // LAYOUT
    .add_system_set_to_stage(
        CoreStage::PostUpdate,
        SystemSet::new()
            .label(MySystems::LAYOUT)
            .with_system(size_scaling)
            .with_system(position_translation)
    )

    // FOOD
    .add_system_set(
        SystemSet::new()
            .with_run_criteria(FixedTimestep::step(2.0)) // TODO change for eating event
            .with_system(spawn_food)
    )
    .add_event::<GrowthEvent>()
    .add_event::<GameOverEvent>()
    .add_plugins(DefaultPlugins)
    //.add_plugin(EditorPlugin)
    .run();
}

// ================================ CAMERA ================================
fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

// =============================== GAME OVER ===============================
struct GameOverEvent;

// fn red(entity: Entity, commands: &mut Commands) {
//     commands.entity(entity)
//     .remove::<Sprite>()
//     .insert(
//         Sprite {
//             color: Color::RED,
//             ..default()
//         }
//     );
// }

// fn snake_flash(segments: &ResMut<SnakeSegments>, commands: &mut Commands) {
//     for _ in 0..3 {
//         // make segments red
//         for segment in segments.iter() {
//             red(*segment, commands);
//         }
//         // wait 
//         std::thread::sleep(std::time::Duration::from_secs_f32(0.2));
//         // make segments disappear
//         for segment in segments.iter() {
//             commands.entity(*segment).remove::<Sprite>();
//         }
//         // wait
//         std::thread::sleep(std::time::Duration::from_secs_f32(0.2));
//     }         
// }

fn game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    segments_res: ResMut<SnakeSegments>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnakeSegment>>,
)   {
    
    if reader.iter().next().is_some() {
        // delete fruits
        for entity in food.iter() {         
            commands.entity(entity).despawn();
        }

        // flash snake
        // snake_flash(&segments_res, &mut commands);

        // delete snake
        for entity in segments.iter() {
            commands.entity(entity).despawn();
        }

        // wait before respawn
        std::thread::sleep(std::time::Duration::from_secs(1));
        spawn_snake(commands, segments_res);
    }
}

// ================================ FOOD ================================
#[derive(Component)]
struct Food;

#[derive(Default, Deref, DerefMut)]
struct NumEaten(u8);

fn spawn_food(mut commands: Commands, positions: Query<&Position>) {

    // vec of positions as resource
    // of size arena height * arena width
    // NO! OVER ENGINEERING! THERES BARELY ANY GRID SQUARES, JUST ITERATE THROUGH SEGMENTS!

    // choose position x and y not occupied by any snake segment
    let mut x = (random::<u32>() % (ARENA_WIDTH as u32)) as i32;
    let mut y = (random::<u32>() % (ARENA_HEIGHT as u32)) as i32;
    let mut overlap: bool = !(positions.is_empty());
    while overlap == true {
        overlap = false;
        for pos in positions.iter() {
            if pos.x == x && pos.y == y {
                x = (random::<u32>() % (ARENA_WIDTH as u32)) as i32;
                y = (random::<u32>() % (ARENA_HEIGHT as u32)) as i32;
                // info!("overlap");
                overlap = true;
            }
        }
    }
    // info!("no overlap");
    commands.spawn_bundle(
        SpriteBundle {
            sprite: Sprite {
                color: FOOD_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Food)
        .insert(Position {
            x: x,
            y: y,
        })
        .insert(Size::square(FOOD_SIZE));
}


// ================================ SNAKE ================================

#[derive(Component)]
struct SnakeHead {
    direction: Direction
}

#[derive(Component)]
struct SnakeSegment;

#[derive(Default, Deref, DerefMut)]
struct SnakeSegments(Vec<Entity>);

#[derive(Default)]
struct LastTailPosition(Option<Position>);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x
        }
    }
}

#[derive(PartialEq, Copy, Clone, Default, Debug)]
enum Direction {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}


// startup system, spawn the snake
fn spawn_snake(
    mut commands: Commands, 
    mut segments: ResMut<SnakeSegments>,
)   {

    // spawn the snake, with 2 segments: the head and one tail segment
    *segments = SnakeSegments(vec![
        
        // head
        commands.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_HEAD_COLOR,
                ..default()
            },
            transform: Transform {
                scale: Vec3::new(SNAKE_SIZE, SNAKE_SIZE, SNAKE_SIZE),
                ..default()
            },
            ..default()
            })
            .insert(SnakeHead { direction: Direction::Left })
            .insert(SnakeSegment)
            .insert(Position { x: 3, y: 3 })
            .insert(Size::square(SNAKE_SIZE))
            .id(),

        // body
        spawn_segment(commands, Position {x: 3, y: 2})
    ]);
 
}

// helper function to spawn segment of snake, either at setup or when eating
fn spawn_segment(mut commands: Commands, position: Position) -> Entity {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_SEGMENT_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(SnakeSegment)
        .insert(position)
        .insert(Size::square(SEGMENT_SIZE))
        .id()
}

// movement system
fn snake_movement(
    segments: ResMut<SnakeSegments>,
    last_input: Res<LastUserInput>,
    mut heads: Query<(Entity, &mut SnakeHead)>,
    mut positions: Query<&mut Position>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_writer: EventWriter<GameOverEvent>
)   {
    if let Some((head_entity, mut head)) = heads.iter_mut().next() {

        let segment_positions = segments
            .iter()
            .map(|entity| *positions.get_mut(*entity).unwrap())
            .collect::<Vec<Position>>();

        let mut head_pos = positions.get_mut(head_entity).unwrap();

        *last_tail_position = LastTailPosition(Some(*segment_positions.last().unwrap()));

        let mut dir: Direction = last_input.0;
        if dir == head.direction.opposite() {
            dir = head.direction;
        }

        head.direction = dir;
        match dir {
            Direction::Left => {
                head_pos.x -= STEP_SIZE;
                if head_pos.x < 0 {
                    head_pos.x = ARENA_WIDTH as i32 - 1;
                }
            }
            Direction::Right => {
                head_pos.x += STEP_SIZE;
                if head_pos.x >= ARENA_WIDTH as i32 {
                    head_pos.x = 0;
                }
            }
            Direction::Down => {
                head_pos.y -= STEP_SIZE;
                if head_pos.y < 0 {
                    head_pos.y = ARENA_HEIGHT as i32 - 1;
                }
            }
            Direction::Up => {
                head_pos.y += STEP_SIZE;
                if head_pos.y >= ARENA_HEIGHT as i32 {
                    head_pos.y = 0;
                }
            }
        };

        // info!("{}, {}", head_pos.x, head_pos.y);

        if segment_positions.contains(&head_pos) {
            game_over_writer.send(GameOverEvent);
        }

        segment_positions
            .iter()
            .zip(segments.iter().skip(1))
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            })
    }
}

#[derive(Default)]
struct LastUserInput(Direction);

fn key_input(mut key_evr: EventReader<KeyboardInput>, mut last_input: ResMut<LastUserInput>) {
    for ev in key_evr.iter() {
        match ev.state {
            ButtonState::Pressed => {
                info!("Pressed: {:?}", ev.key_code.unwrap());
                last_input.0 = match ev.key_code.unwrap() {
                    KeyCode::Left => Direction::Left,
                    KeyCode::Right => Direction::Right,
                    KeyCode::Up => Direction::Up,
                    KeyCode::Down => Direction::Down,
                    _ => last_input.0,
                };
            }
            ButtonState::Released => ()
        };
    }
}

struct GrowthEvent;

// growth system
fn snake_growth(
    commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: EventReader<GrowthEvent>
)   {
    if growth_reader.iter().next().is_some() {
        segments.push(spawn_segment(commands, last_tail_position.0.unwrap()));
    }
}


// eating system
fn snake_eating(
    mut commands: Commands,
    mut growth_w: EventWriter<GrowthEvent>,
    mut num_eaten: ResMut<NumEaten>,
    food_pos: Query<(Entity, &Position), With<Food>>,
    head_pos: Query<&Position, With<SnakeHead>>
)   {
    for pos in head_pos.iter(){
        for (ent, food) in food_pos.iter() {
            if food == pos {
                commands.entity(ent).despawn();
                num_eaten.0 += 1;
                growth_w.send(GrowthEvent);
            }
        }
    }
}

// ================================ LAYOUT ==========================================


// system to scale the snake to the correct size
fn size_scaling(
    windows: Res<Windows>,
    mut q: Query<(&Size, &mut Transform)>,
    mut exit: EventWriter<AppExit>
)   {
    match windows.get_primary() {
        None => exit.send(AppExit),
        Some(window) => {
            for (sprite_size, mut transform) in q.iter_mut() {
                transform.scale = Vec3::new(
                    sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
                    sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
                    1.0
                );
            }
        }
    }

}

// system to position the snake in the arena
fn position_translation(
    windows: Res<Windows>,
    mut q: Query<(&Position, &mut Transform)>,
    mut exit: EventWriter<AppExit>
)   {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    match windows.get_primary() {
        None => exit.send(AppExit),
        Some(window) => {
            for (pos, mut transform) in q.iter_mut() {
                transform.translation = Vec3::new(
                    convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
                    convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
                    1.0
                );
            }
        }
    }   
}
