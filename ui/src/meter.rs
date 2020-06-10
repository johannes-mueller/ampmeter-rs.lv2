
use pugl_ui::ui::*;
use pugl_ui::*;
use pugl_ui::widget::*;
use pugl_sys::*;

pub struct Meter {
    stub: WidgetStub,
    width: f64,

    value: f32,
    min_value: f32,
    max_value: f32
}

fn draw_bar_part(value: f64, lower_limit: f64, min: f64, max: f64,
		 pos: Coord, w: f64, h: f64, color: (f64, f64, f64),
		 cr: &cairo::Context) -> f64 {
    if value <= lower_limit {
	return value;
    }

    let lower = (lower_limit - min) * h/(max-min);
    let height = (value-lower_limit) * h/(max-min);
    let (r,g,b) = color;
    cr.set_source_rgb(r,g,b);
    cr.rectangle(pos.x, pos.y + lower, w, height);
    cr.fill();

    lower_limit
}

impl Widget for Meter {

    fn exposed(&self, _exposed: &ExposeArea, cr: &cairo::Context) {
	let size = self.size();
	let w = size.w;
	let h = -size.h;
	let pos = self.pos() + Coord { x: 0.0, y: -h };
	let mut value = self.value as f64;

	let min_value = self.min_value as f64;
	let max_value = self.max_value as f64;

	value = draw_bar_part(value, 0.0, min_value, max_value, pos, w,h, (1.0, 0.0, 0.0), cr);
	value = draw_bar_part(value, -18.0, min_value, max_value, pos, w,h, (1.0, 1.0, 0.0), cr);
	draw_bar_part(value, min_value, min_value, max_value, pos, w,h, (0.0, 1.0, 0.0), cr);

	cr.set_source_rgb(0.7, 0.7, 0.7);
	cr.set_line_width(1.0);
	cr.rectangle(pos.x, pos.y, w, h);
	cr.stroke();
    }

    fn min_size(&self) -> Size {
	Size { w: self.width, h: 2.0 * self.width }
    }

    fn stub (&self) -> &WidgetStub {
        &self.stub
    }
    fn stub_mut (&mut self) -> &mut WidgetStub {
        &mut self.stub
    }
    fn height_expandable(&self) -> bool { true }
}

impl Meter {
    pub fn new(min_value: f32, max_value: f32) -> Box<Meter> {
	Box::new( Meter { stub: WidgetStub::default(),
			  min_value, max_value,
			  value: min_value,
			  width: 25.0
	})
    }
    pub fn set_value(&mut self, v: f32) {
	self.value = match v {
	    v if v < self.min_value => self.min_value,
	    v if v > self.max_value => self.max_value,
	    _ => v
	};
	self.ask_for_repaint();
    }
}
