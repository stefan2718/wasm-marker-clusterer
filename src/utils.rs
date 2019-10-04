extern crate googleprojection;
use self::googleprojection::{from_ll_to_subpixel, from_pixel_to_ll};

use structs::bounds::Bounds;

pub fn calculate_extended_bounds(bounds: &Bounds, zoom: usize, grid_size: f64) -> Bounds {
    let mut north_east_pix = from_ll_to_subpixel(&(bounds.east, bounds.north), zoom).unwrap();
    let mut south_west_pix = from_ll_to_subpixel(&(bounds.west, bounds.south), zoom).unwrap();

    north_east_pix.0 += grid_size;
    north_east_pix.1 -= grid_size;

    south_west_pix.0 -= grid_size;
    south_west_pix.1 += grid_size;
    
    // println!("ne0 {}, ne1 {}, sw0 {}, sw1 {}", north_east_pix.0, north_east_pix.1, south_west_pix.0, south_west_pix.1);
    let north_east_latlng = from_pixel_to_ll(&(north_east_pix.0, north_east_pix.1), zoom).unwrap();
    let south_west_latlng = from_pixel_to_ll(&(south_west_pix.0, south_west_pix.1), zoom).unwrap();

    Bounds {
        north: north_east_latlng.1,
        east: north_east_latlng.0,
        south: south_west_latlng.1,
        west: south_west_latlng.0,
    }
}
