use std::{
    iter::{self, Once},
    result,
};

use plotters::{
    element::{BackendCoordOnly, CoordMapper, Drawable, PointCollection},
    prelude::{Circle, Cross, TriangleMarker},
    style::ShapeStyle,
};
use plotters_backend::{DrawingBackend, DrawingErrorKind};

#[derive(Debug)]
pub enum MarkerKind {
    Triangle,
    Circle,
    Cross,
}

impl MarkerKind {
    pub const COUNT: usize = 3;
}

#[derive(Debug)]
pub struct Marker<Coord> {
    center: Coord,
    style: ShapeStyle,
    kind: MarkerKind,
}

impl<DB, Coord> Drawable<DB> for Marker<Coord>
where
    DB: DrawingBackend,
{
    fn draw<I>(
        &self,
        pos: I,
        backend: &mut DB,
        parent_dim: (u32, u32),
    ) -> result::Result<(), DrawingErrorKind<<DB as DrawingBackend>::ErrorType>>
    where
        I: Iterator<Item = <BackendCoordOnly as CoordMapper>::Output>,
    {
        match self.kind {
            MarkerKind::Triangle => {
                TriangleMarker::new(&self.center, 5, self.style).draw(pos, backend, parent_dim)
            }
            MarkerKind::Circle => {
                Circle::new(&self.center, 5, self.style).draw(pos, backend, parent_dim)
            }
            MarkerKind::Cross => {
                Cross::new(&self.center, 5, self.style).draw(pos, backend, parent_dim)
            }
        }
    }
}

impl<'a, Coord> PointCollection<'a, Coord> for &'a Marker<Coord>
where
    Coord: 'a,
{
    type Point = &'a Coord;
    type IntoIter = Once<&'a Coord>;
    fn point_iter(self) -> Once<&'a Coord> {
        iter::once(&self.center)
    }
}

impl<Coord> Marker<Coord> {
    pub fn new<S>(kind: MarkerKind, coord: Coord, style: S) -> Self
    where
        S: Into<ShapeStyle>,
    {
        Self {
            center: coord,
            style: style.into(),
            kind,
        }
    }
}
