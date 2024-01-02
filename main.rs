use std::f32::MAX_EXP;
use std::{thread, time};
use rand::prelude::*;
use raylib::prelude::*;
use raylib::ffi::GetScreenWidth;
use raylib::ffi::GetScreenHeight;
use raylib::ffi::SetConfigFlags;
use raylib::ffi::ConfigFlags::FLAG_WINDOW_RESIZABLE;
use raylib::ffi::KeyboardKey::*;

const K: i32 = 4;
const SAMPLE_R: f32 = 4.0;
const MEAN_R: f32 = SAMPLE_R*2.0;
const MIN_X: f32 = -20.0;
const MAX_X: f32 = 20.0;
const MIN_Y: f32 = -20.0;
const MAX_Y: f32 = 20.0;
const PI: f32 = 3.1415926535897;
const COLORS: [Color; 9] = [Color::YELLOW, Color::PINK, Color::BROWN, Color::GREEN, Color::LIME, Color::SKYBLUE, Color::PURPLE, Color::VIOLET, Color::BEIGE];


fn gen_float() -> f32{
    let mut rng = rand::thread_rng();
    return rng.gen();
}

fn lerp(min: f32, max: f32, rand: f32) -> f32{
    return rand*(max-min) + min;
}

fn vector_sum(v1: Vector2, v2: Vector2) -> Vector2{
    return Vector2{x: v1.x+v2.x, y: v1.y - v2.y};
}

fn vector_subtract(v1: Vector2, v2: Vector2) -> Vector2{
    return Vector2{x: v1.x - v2.x, y: v1.y - v2.y};
}

fn vector_length(v: Vector2) -> f32{
    return (v.x*v.x)+(v.y*v.y);
}



fn generate_cluster(center: Vector2, radius: f32, count: i32, samples: &mut Vec<Vector2>){
    for i in 0..count{
        let angle: f32 = gen_float()*2.0*PI;
        let mag: f32 = gen_float();
        let sample = Vector2 {x: center.x + angle.cos()*mag*radius, y: center.y + angle.sin()*mag*radius};
        samples.push(sample);
    }
}

fn project_sample(sample: Vector2) -> Vector2{
    let nx: f32 = (sample.x - MIN_X)/(MAX_X - MIN_X);
    let ny: f32 = (sample.y - MIN_Y)/(MAX_Y - MIN_Y);
    let w: f32 = unsafe{GetScreenWidth() as f32};
    let h: f32 = unsafe{GetScreenHeight() as f32};

    return Vector2{x: nx*w, y: h-ny*h};
}

fn regenerate_cluster(cluster: &mut Vec<Vector2>, means: &mut Vec<Vector2>) -> (){

    generate_cluster(Vector2{x:0.0,y:0.0}, 10.0, 100, cluster);
    generate_cluster(Vector2{x: MIN_X*0.5, y: MAX_Y*0.5}, 5.0, 100, cluster);
    generate_cluster(Vector2{x: MAX_X*0.5, y: MAX_Y*0.5}, 5.0, 100, cluster);
    generate_cluster(Vector2{x: MIN_X*0.5, y: MIN_Y*0.5}, 5.0, 100, cluster);
    generate_cluster(Vector2{x: MAX_X*0.5, y: MIN_Y*0.5}, 5.0, 100, cluster);

    for _ in 0..K{
        means.push(Vector2{x: lerp(MIN_X, MAX_X, gen_float()), y: lerp(MIN_Y, MAX_Y, gen_float())});
    }
}

fn recluster(cluster: &Vec<Vector2>, clusters: &mut Vec<Vec<Vector2>>, means: &Vec<Vector2>){
    for _ in 0..K{
        clusters.push(Vec::<Vector2>::new());
    }
    for i in 0..cluster.len(){
        let p: Vector2 = cluster[i];
        let mut k: usize = usize::MIN;
        let mut s: f32 = MAX_EXP as f32;
        for j in 0..K{
            let m: Vector2 = means[j as usize];
            let sm = vector_length(vector_subtract(p, m));
            if sm < s{
                s = sm;
                k = j as usize;
            }
        }
        clusters[k].push(p);
    }
}

fn update_means(clusters: &Vec<Vec<Vector2>>, means: &mut Vec<Vector2>){
    for i in 0..K{
        if clusters[i as usize].len() as i32 > 0{
            means[i as usize] = Vector2{x: 0.0, y: 0.0};
            for j in 0..clusters[i as usize].len(){
                means[i as usize] = vector_sum(means[i as usize], clusters[i as usize][j]);
            }
            means[i as usize].x /= clusters[i as usize].len() as f32;
            means[i as usize].y /= clusters[i as usize].len() as f32;
        }
        else{
            means[i as usize].x = lerp(MIN_X, MAX_X, gen_float());
            means[i as usize].y = lerp(MIN_Y, MAX_Y, gen_float());
        }
    }
}

fn main(){
    let mut clusters: Vec<Vec<Vector2>> = vec![];
    let mut means: Vec<Vector2> = vec![];
    let mut cluster: Vec<Vector2> = vec![];
    let mut start: bool = false;

    for _ in 0..K{
        clusters.push(Vec::<Vector2>::new());
    }

    cluster = vec![];
    regenerate_cluster(&mut cluster, &mut means);

    unsafe{SetConfigFlags(FLAG_WINDOW_RESIZABLE as u32)};
    let (mut rl, thread) = raylib::init()
        .size(800, 600)
        .title("K means")
        .build();

    recluster(&cluster, &mut clusters, &means);


    while !rl.window_should_close(){
        if rl.is_key_pressed(KEY_R){
            cluster = vec![];
            means = vec![];
            regenerate_cluster(&mut cluster, &mut means);
            clusters = vec![];
            recluster(&cluster, &mut clusters, &means);
        }
        if rl.is_key_pressed(KEY_SPACE){
            if !start{
                start = true;
            }
            else{
                start = false;
            }
        }
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::get_color(0x181818AA));
        if start{
            update_means(&clusters, &mut means);
            clusters = vec![];
            recluster(&cluster, &mut clusters, &means);
            thread::sleep(time::Duration::from_millis(500));
        }
        for i in 0..cluster.len(){
            d.draw_circle_v(project_sample(cluster[i]), SAMPLE_R, Color::RED);
        }
        for i in 0..K{
            let color: Color = COLORS[i as usize %COLORS.len()];
            for j in 0..clusters[i as usize].len(){
                d.draw_circle_v(project_sample(clusters[i as usize][j]), SAMPLE_R, color);
            }
            d.draw_circle_v(project_sample(means[i as usize]), MEAN_R, color);
        }
    }
}
