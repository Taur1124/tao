// Copyright 2014-2021 The winit contributors
// Copyright 2021-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

//! UI scaling is important, so read the docs for this module if you don't want to be confused.
//!
//! ## Why should I care about UI scaling?
//!
//! Modern computer screens don't have a consistent relationship between resolution and size.
//! 1920x1080 is a common resolution for both desktop and mobile screens, despite mobile screens
//! normally being less than a quarter the size of their desktop counterparts. What's more, neither
//! desktop nor mobile screens are consistent resolutions within their own size classes - common
//! mobile screens range from below 720p to above 1440p, and desktop screens range from 720p to 5K
//! and beyond.
//!
//! Given that, it's a mistake to assume that 2D content will only be displayed on screens with
//! a consistent pixel density. If you were to render a 96-pixel-square image on a 1080p screen,
//! then render the same image on a similarly-sized 4K screen, the 4K rendition would only take up
//! about a quarter of the physical space as it did on the 1080p screen. That issue is especially
//! problematic with text rendering, where quarter-sized text becomes a significant legibility
//! problem.
//!
//! Failure to account for the scale factor can create a significantly degraded user experience.
//! Most notably, it can make users feel like they have bad eyesight, which will potentially cause
//! them to think about growing elderly, resulting in them having an existential crisis. Once users
//! enter that state, they will no longer be focused on your application.
//!
//! ## How should I handle it?
//!
//! The solution to this problem is to account for the device's *scale factor*. The scale factor is
//! the factor UI elements should be scaled by to be consistent with the rest of the user's system -
//! for example, a button that's normally 50 pixels across would be 100 pixels across on a device
//! with a scale factor of `2.0`, or 75 pixels across with a scale factor of `1.5`.
//!
//! Many UI systems, such as CSS, expose DPI-dependent units like [points] or [picas]. That's
//! usually a mistake, since there's no consistent mapping between the scale factor and the screen's
//! actual DPI. Unless you're printing to a physical medium, you should work in scaled pixels rather
//! than any DPI-dependent units.
//!
//! ### Position and Size types
//!
//! Tao's `Physical(Position|Size)` types correspond with the actual pixels on the device, and the
//! `Logical(Position|Size)` types correspond to the physical pixels divided by the scale factor.
//! All of Tao's functions return physical types, but can take either logical or physical
//! coordinates as input, allowing you to use the most convenient coordinate system for your
//! particular application.
//!
//! Tao's position and size types types are generic over their exact pixel type, `P`, to allow the
//! API to have integer precision where appropriate (e.g. most window manipulation functions) and
//! floating precision when necessary (e.g. logical sizes for fractional scale factors and touch
//! input). If `P` is a floating-point type, please do not cast the values with `as {int}`. Doing so
//! will truncate the fractional part of the float, rather than properly round to the nearest
//! integer. Use the provided `cast` function or `From`/`Into` conversions, which handle the
//! rounding properly. Note that precision loss will still occur when rounding from a float to an
//! int, although rounding lessens the problem.
//!
//! ### Events
//!
//! Tao will dispatch a [`ScaleFactorChanged`](crate::event::WindowEvent::ScaleFactorChanged)
//! event whenever a window's scale factor has changed. This can happen if the user drags their
//! window from a standard-resolution monitor to a high-DPI monitor, or if the user changes their
//! DPI settings. This gives you a chance to rescale your application's UI elements and adjust how
//! the platform changes the window's size to reflect the new scale factor. If a window hasn't
//! received a [`ScaleFactorChanged`](crate::event::WindowEvent::ScaleFactorChanged) event,
//! then its scale factor is `1.0`.
//!
//! ## How is the scale factor calculated?
//!
//! Scale factor is calculated differently on different platforms:
//!
//! - **Windows:** On Windows 8 and 10, per-monitor scaling is readily configured by users from the
//!   display settings. While users are free to select any option they want, they're only given a
//!   selection of "nice" scale factors, i.e. 1.0, 1.25, 1.5... on Windows 7, the scale factor is
//!   global and changing it requires logging out. See [this article][windows_1] for technical
//!   details.
//! - **macOS:** Recent versions of macOS allow the user to change the scaling factor for certain
//!   displays. When this is available, the user may pick a per-monitor scaling factor from a set
//!   of pre-defined settings. All "retina displays" have a scaling factor above 1.0 by default but
//!   the specific value varies across devices.
//! - **Linux:** TODO
//! - **iOS:** Scale factors are set by Apple to the value that best suits the device, and range
//!   from `1.0` to `3.0`. See [this article][apple_1] and [this article][apple_2] for more
//!   information.
//! - **Android:** Scale factors are set by the manufacturer to the value that best suits the
//!   device, and range from `1.0` to `4.0`. See [this article][android_1] for more information.
//!
//! [points]: https://en.wikipedia.org/wiki/Point_(typography)
//! [picas]: https://en.wikipedia.org/wiki/Pica_(typography)
//! [windows_1]: https://docs.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development-on-windows
//! [apple_1]: https://developer.apple.com/library/archive/documentation/DeviceInformation/Reference/iOSDeviceCompatibility/Displays/Displays.html
//! [apple_2]: https://developer.apple.com/design/human-interface-guidelines/macos/icons-and-images/image-size-and-resolution/
//! [android_1]: https://developer.android.com/training/multiscreen/screendensities

pub trait Pixel: Copy + Into<f64> {
  fn from_f64(f: f64) -> Self;
  fn cast<P: Pixel>(self) -> P {
    P::from_f64(self.into())
  }
}

macro_rules! pixel_int_impl {
  ($($t:ty),*) => {$(
      impl Pixel for $t {
          fn from_f64(f: f64) -> Self {
              f.round() as $t
          }
      }
  )*}
}

pixel_int_impl!(u8, u16, u32, i8, i16, i32);

impl Pixel for f32 {
  fn from_f64(f: f64) -> Self {
    f as f32
  }
}

impl Pixel for f64 {
  fn from_f64(f: f64) -> Self {
    f
  }
}

macro_rules! from_impls {
  ($t:ident, $a:ident, $(,)? ) => {
    impl<P: Pixel> From<P> for $t<P> {
      fn from($a: P) -> Self {
        Self::new($a.cast())
      }
    }
  };
  ($t:ident, $a:ident, $b:ident$(,)? ) => {
    impl<P: Pixel, X: Pixel> From<(X, X)> for $t<P> {
      fn from(($a, $b): (X, X)) -> Self {
        Self::new($a.cast(), $b.cast())
      }
    }

    impl<P: Pixel, X: Pixel> From<$t<P>> for (X, X) {
      fn from(p: $t<P>) -> Self {
        (p.$a.cast(), p.$b.cast())
      }
    }

    impl<P: Pixel, X: Pixel> From<[X; 2]> for $t<P> {
      fn from([$a, $b]: [X; 2]) -> Self {
        Self::new($a.cast(), $b.cast())
      }
    }

    impl<P: Pixel, X: Pixel> From<$t<P>> for [X; 2] {
      fn from(p: $t<P>) -> Self {
        [p.$a.cast(), p.$b.cast()]
      }
    }
  };
}

/// Checks that the scale factor is a normal positive `f64`.
///
/// All functions that take a scale factor assert that this will return `true`. If you're sourcing scale factors from
/// anywhere other than tao, it's recommended to validate them using this function before passing them to tao;
/// otherwise, you risk panics.
#[inline]
pub fn validate_scale_factor(scale_factor: f64) -> bool {
  scale_factor.is_sign_positive() && scale_factor.is_normal()
}

macro_rules! dpi_type {
  (
      $(let $a:ident;)*

      $(#[$logical_meta:meta])*
      pub struct $LogicalType:ident;
      $(#[$physical_meta:meta])*
      pub struct $PhysicalType:ident;
      $(#[$unified_meta:meta])*
      pub enum $UnifiedType:ident {
          Physical($unified_physical:ty),
          Logical($unified_logical:ty),
      }
  ) => {
      $(#[$logical_meta])*
      #[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash, PartialOrd, Ord)]
      #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
      pub struct $LogicalType<P> {
          $(pub $a: P,)*
      }

      impl<P> $LogicalType<P> {
          #[inline]
          pub const fn new($($a: P,)*) -> Self {
              $LogicalType { $($a,)* }
          }
      }

      impl<P: Pixel> $LogicalType<P> {
          #[inline]
          pub fn from_physical<T: Into<$PhysicalType<X>>, X: Pixel>(
              physical: T,
              scale_factor: f64,
          ) -> Self {
              physical.into().to_logical(scale_factor)
          }

          #[inline]
          pub fn to_physical<X: Pixel>(&self, scale_factor: f64) -> $PhysicalType<X> {
              assert!(validate_scale_factor(scale_factor));
              $(let $a = self.$a.into() * scale_factor;)*
              $PhysicalType::new($($a,)*).cast()
          }

          #[inline]
          pub fn cast<X: Pixel>(&self) -> $LogicalType<X> {
              $LogicalType {
                  $($a: self.$a.cast(),)*
              }
          }
      }

      from_impls!($LogicalType, $($a,)*);

      $(#[$physical_meta])*
      #[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash, PartialOrd, Ord)]
      #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
      pub struct $PhysicalType<P> {
        $(pub $a: P,)*
      }

      impl<P> $PhysicalType<P> {
          #[inline]
          pub const fn new($($a: P,)*) -> Self {
              $PhysicalType { $($a,)* }
          }
      }

      impl<P: Pixel> $PhysicalType<P> {
          #[inline]
          pub fn from_logical<T: Into<$LogicalType<X>>, X: Pixel>(
              logical: T,
              scale_factor: f64,
          ) -> Self {
              logical.into().to_physical(scale_factor)
          }

          #[inline]
          pub fn to_logical<X: Pixel>(&self, scale_factor: f64) -> $LogicalType<X> {
              assert!(validate_scale_factor(scale_factor));
              $(let $a = self.$a.into() / scale_factor;)*
              $LogicalType::new($($a,)*).cast()
          }

          #[inline]
          pub fn cast<X: Pixel>(&self) -> $PhysicalType<X> {
              $PhysicalType {
                  $($a: self.$a.cast(),)*
              }
          }
      }

      from_impls!($PhysicalType, $($a,)*);

      $(#[$unified_meta])*
      #[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
      #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
      pub enum $UnifiedType {
          Physical($unified_physical),
          Logical($unified_logical),
      }

      impl $UnifiedType {
          pub fn new<S: Into<$UnifiedType>>(val: S) -> $UnifiedType {
              val.into()
          }

          pub fn to_logical<P: Pixel>(&self, scale_factor: f64) -> $LogicalType<P> {
              match *self {
                  $UnifiedType::Physical(val) => val.to_logical(scale_factor),
                  $UnifiedType::Logical(val) => val.cast(),
              }
          }

          pub fn to_physical<P: Pixel>(&self, scale_factor: f64) -> $PhysicalType<P> {
              match *self {
                  $UnifiedType::Physical(val) => val.cast(),
                  $UnifiedType::Logical(val) => val.to_physical(scale_factor),
              }
          }

          $(pub fn $a(&self) -> PixelUnit {
            match *self {
              $UnifiedType::Physical(any) => PixelUnit::Physical(any.$a.into()),
              $UnifiedType::Logical(any) => PixelUnit::Logical(any.$a.into()),
            }
          })*
      }

      impl<P: Pixel> From<$PhysicalType<P>> for $UnifiedType {
          #[inline]
          fn from(val: $PhysicalType<P>) -> $UnifiedType {
              $UnifiedType::Physical(val.cast())
          }
      }

      impl<P: Pixel> From<$LogicalType<P>> for $UnifiedType {
          #[inline]
          fn from(val: $LogicalType<P>) -> $UnifiedType {
              $UnifiedType::Logical(val.cast())
          }
      }
  };
}

dpi_type! {
  let value;

  /// A logical pixel.
  pub struct LogicalPixel;
  /// A physical pixel.
  pub struct PhysicalPixel;
  /// A pixel that's either physical or logical.
  pub enum PixelUnit {
      Physical(PhysicalPixel<i32>),
      Logical(LogicalPixel<f64>),
  }
}

impl PixelUnit {
  /// Represents a minimum logical unit of `0`
  pub const MIN: PixelUnit = PixelUnit::Logical(LogicalPixel::new(0.0));
  /// Represents a maximum logical unit that is equal to [`f64::MAX`]
  pub const MAX: PixelUnit = PixelUnit::Logical(LogicalPixel::new(f64::MAX));
}

impl From<u32> for PhysicalPixel<i32> {
  fn from(value: u32) -> Self {
    Self::new(value.cast())
  }
}

dpi_type! {
  let x;
  let y;

  /// A position represented in logical pixels.
  ///
  /// The position is stored as floats, so please be careful. Casting floats to integers truncates the
  /// fractional part, which can cause noticable issues. To help with that, an `Into<(i32, i32)>`
  /// implementation is provided which does the rounding for you.
  pub struct LogicalPosition;
  /// A position represented in physical pixels.
  pub struct PhysicalPosition;
  /// A position that's either physical or logical.
  pub enum Position {
      Physical(PhysicalPosition<i32>),
      Logical(LogicalPosition<f64>),
  }
}

dpi_type! {
  let width;
  let height;

  /// A size represented in logical pixels.
  pub struct LogicalSize;
  /// A size represented in physical pixels.
  pub struct PhysicalSize;
  /// A size that's either physical or logical.
  pub enum Size {
      Physical(PhysicalSize<u32>),
      Logical(LogicalSize<f64>),
  }
}

impl Size {
  pub fn clamp<S: Into<Size>>(desired_size: S, min: S, max: S, scale_factor: f64) -> Size {
    let (desired_size, min, max) = (
      desired_size.into().to_physical::<f64>(scale_factor),
      min.into().to_physical::<f64>(scale_factor),
      max.into().to_physical::<f64>(scale_factor),
    );

    let clamp = |desired_size: f64, min: f64, max: f64| {
      if desired_size < min {
        min
      } else if desired_size > max {
        max
      } else {
        desired_size
      }
    };

    let width = clamp(desired_size.width, min.width, max.width);
    let height = clamp(desired_size.height, min.height, max.height);

    PhysicalSize::new(width, height).into()
  }
}
