use plotters::prelude::*;
use rand::Rng;
use std::collections::HashMap;
use crate::graph::component_functions::count_components;

pub fn show_aggregation(points:&Vec<f64>,filename:&str) -> Result<(), Box<dyn std::error::Error>> {
    // Create the drawing area, using the provided filename
    let root = BitMapBackend::new(filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    // Change Title Name Later
    let root_area = root.titled("Connectivity Progress by Component", ("sans-serif", 40))?;
    
    // Create the chart builder with custom x-axis range (allowing negative values)
    let mut cc = ChartBuilder::on(&root_area)
        .margin(5)
        .set_all_label_area_size(50)
        .build_cartesian_2d(-5..points.len() as isize, 0.0..1.05)?;  // Adjust x-range to allow negative values

    // Configure mesh and labels for y-axis
    cc.configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .disable_mesh()
        .y_label_formatter(&|v: &f64| format!("{:.1}", v))
        .draw()?;

    // Draw the line series
    cc.draw_series(LineSeries::new(
        points.iter().enumerate().map(|(index, &value)| {
            (index as isize, value) // Map x to allow negative values
        }),
        &BLUE,
    ))?;

    Ok(())
}

pub fn get_graph_dimensions(
    component: &Vec<Option<usize>>, 
    num_components: usize, 
    drawing_bounds: (i32, i32, i32, i32),
    biggest_circle: f64,
    grid_size: f64, // Size of each cell in the grid
) -> HashMap<usize, ((f64, f64), f64)> {
    let (x_min, x_max, y_min, y_max) = drawing_bounds;
    let component_counts = count_components(component, num_components + 1);
    let total: usize = component_counts.iter().sum();
    let mut rng = rand::thread_rng();
    let mut component_sorted_tuples: Vec<(usize, usize)> = component_counts
        .iter()
        .enumerate()
        .map(|(index, value)| (index, *value))
        .collect();
    component_sorted_tuples.sort_by(|a, b| b.1.cmp(&a.1));
    println!("{:?}",component_sorted_tuples);
    let mut current_x = x_min as f64;
    let total_x_space = (x_max - x_min) as f64;

    let mut current_y = y_min as f64;
    let total_y_space = (y_max - y_min) as f64;

    let mut range_hash = HashMap::<usize, ((f64, f64), f64)>::new();
    let mut grid: HashMap<(usize, usize), Vec<(f64, f64, f64)>> = HashMap::new();

    for (index, value) in component_sorted_tuples {
        let percentage_needed = value as f64 / total as f64;

        // Calculate radius based on percentage size
        let min_radius = 50.0;
        let max_radius = (total_x_space.min(total_y_space)) / biggest_circle;
        let circle_radius = (percentage_needed.sqrt() * max_radius).max(min_radius);

        // Determine cluster center
        let mut center_x = current_x + circle_radius;
        let mut center_y = current_y + circle_radius;

        // Spatial checking to resolve overlap
        let mut overlap = true;
        while overlap {
            overlap = false;

            // Check positions in the grid for potential overlap
            let cell_x = (center_x / grid_size).floor() as usize;
            let cell_y = (center_y / grid_size).floor() as usize;

            if let Some(neighbors) = grid.get(&(cell_x, cell_y)) {
                for &(placed_x, placed_y, placed_radius) in neighbors {
                    let distance = ((center_x - placed_x).powi(2) + (center_y - placed_y).powi(2)).sqrt();
                    if distance < (circle_radius + placed_radius) {
                        overlap = true;
                        break;
                    }
                }
            }

            if overlap {
                // Randomly adjust position if there was an overlap
                center_x = rng.gen_range(x_min as f64..x_max as f64);
                center_y = rng.gen_range(y_min as f64..y_max as f64);
            }
        }

        // Store the new position and radius in the hash map
        range_hash.insert(index, ((center_x, center_y), circle_radius));
        let cell_x = (center_x / grid_size).floor() as usize;
        let cell_y = (center_y / grid_size).floor() as usize;
        grid.entry((cell_x, cell_y)).or_default().push((center_x, center_y, circle_radius));

        // Move horizontally for the next component with jitter
        current_x += circle_radius * rng.gen_range(2.0..4.0);

        // Step down the Y-axis dynamically for the next component with jitter
        current_y += circle_radius * rng.gen_range(2.0..3.0) * (1.0 + (1.0 - percentage_needed.sqrt()));

        // Check bounds
        if current_x + circle_radius > x_max as f64 {
            current_x = x_min as f64 + rng.gen_range(0.0..100.0);
        }

        if current_y + circle_radius > y_max as f64 {
            current_y = y_min as f64 + rng.gen_range(0.0..100.0);
        }
    }

    range_hash
}


pub fn interpolate_color(start_color: (u8, u8, u8), end_color: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    let (r_start, g_start, b_start) = start_color;
    let (r_end, g_end, b_end) = end_color;

    let r = (r_start as f64 + t * (r_end as f64 - r_start as f64)) as u8;
    let g = (g_start as f64 + t * (g_end as f64 - g_start as f64)) as u8;
    let b = (b_start as f64 + t * (b_end as f64 - b_start as f64)) as u8;

    (r, g, b)
}

pub fn get_color_from_gradient(index: usize, total: usize) -> RGBAColor {
    // Define the start and end colors (dark blue to teal)
    let dark_blue = (0, 0, 139);  // RGB for dark blue
    let teal = (0, 128, 128);    // RGB for teal

    // Normalize the index to a value between 0.0 and 1.0
    let t = index as f64 / (total - 1) as f64;

    // Interpolate the color based on the normalized value
    let (r,g,b) = interpolate_color(dark_blue, teal, t);
    RGBAColor(r,g,b,1.0)
}
