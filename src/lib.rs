//! # `cint` - `c`olor `int`erop
//!
//! This library provides a lean, minimal, and stable set of types
//! for color interoperation between crates in Rust. Its goal is to serve the same
//! function that [`mint`](https://docs.rs/mint/) provides for (linear algebra) math types.
//! It does not actually provide any conversion, math, etc. for these types, but rather
//! serves as a stable interface that multiple libraries can rely on and then convert
//! to their own internal representations to actually use. It is also `#![no_std]`.
//! [`bytemuck`](https://docs.rs/bytemuck/) impls are provided with the `bytemuck` feature.
//!
//! # How to Use
//!
//! If you have no idea about color management or encoding principles but you want to
//! use this crate in your own, here's a *very basic* rundown.
//!
//! If you have a color that you loaded from an 8-bit format like a PNG, JPG, etc.,
//! **or** if you have a color that you picked from some sort of online color picker
//! or in Photoshop or Aseprite, then what you have is almost certainly an [`EncodedSrgb<u8>`]
//! color. If you have a color that you loaded
//! from a similar format but has floating point values instead of `u8` ints, then you
//! almost certainly instead have a [`EncodedSrgb<f32>`] color.
//!
//! If you "linearized" or performed "inverse gamma correction" on such a color, then you instead
//! might have a [`LinearSrgb<f32>`].
//!
//! If you are more familiar with color encoding, then you'll find a collection of other color spaces
//! represented, as well as the generic [`GenericColor<ComponentTy>`] type which
//! can be used if the color space you wish to use is not represented.
//!
//! The [ColorInterop] trait exists to provide a "canonical" transformation to and from `cint` types.
//! Since it is often possible to convert a color to and from multiple `cint` types, and because of
//! how the Rust type inference system works, it can often be inconvenient to chain together `from`
//! or `into` calls from the [From]/[Into] trait. [ColorInterop] solves this by providing a strongly
//! typed "reference" conversion to/from `cint` types. This way, you can do things like:
//!
//! ```rust
//! let color_crate1 = color_crate2.into_cint().into();
//! // or
//! let color_crate2 = ColorCrate2::from_cint(color_crate1.into());
//! ```
//!
//! which would otherwise be quite inconvenient. **Provider crates** (those that provide their own color
//! types) should implement the relevant [`From`]/[`Into`] implementations to and from `cint` types, and
//! also the [ColorInterop] trait once for each color type. The [`into_cint`][ColorInterop::into_cint] and
//! [`from_cint`][ColorInterop::from_cint] methods will then be provided automatically.
//!
//! ## Colors with alpha channels
//!
//! `cint` provides the [`Alpha<ColorTy>`] and [`PremultipliedAlpha<ColorTy>`]
//! structs, which are generic over the inner `ColorTy`.
//! To represent an [`EncodedSrgb<u8>`] color with a premultiplied alpha component,
//! you'd use [`PremultipliedAlpha<EncodedSrgb<u8>>`]. If, on the other hand, you want to represent
//! an [`Oklab<f32>`] color with an independent alpha component, you'd use [`Alpha<Oklab<f32>>`]
#![no_std]

#[cfg(feature = "bytemuck")]
use bytemuck::{Pod, Zeroable};

/// A trait used to simpify the interface of the [`Alpha`] and [`PremultipliedAlpha`] types.
pub trait ColorStruct {
    type ComponentTy: Clone + Copy;
}

/// A trait that should be implemented by provider crates on their local color types so that you can call
/// `color.to_cint()` and `Color::from_cint(cint_color)`.
///
/// Provider crates should also do relevant `From`/`Into` impls, but [`ColorInterop`] provides a "canonical"
/// transformation to the closest `cint` color type.
pub trait ColorInterop
where
    Self: Into<<Self as ColorInterop>::CintTy>,
{
    type CintTy: Into<Self>;

    /// Convert `self` into its canonical `cint` type.
    fn from_cint(col: Self::CintTy) -> Self {
        col.into()
    }

    /// Create a `Self` from its canonical `cint` type.
    fn into_cint(self) -> Self::CintTy {
        self.into()
    }
}

/// A color with an alpha component.
///
/// The color components and alpha component are completely separate.
#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct Alpha<ColorTy: ColorStruct> {
    /// The contained color, which is completely separate from the `alpha` value.
    pub color: ColorTy,
    /// The alpha component.
    pub alpha: ColorTy::ComponentTy,
}

#[cfg(feature = "bytemuck")]
unsafe impl<ColorTy: ColorStruct + Zeroable> Zeroable for Alpha<ColorTy> {}
#[cfg(feature = "bytemuck")]
unsafe impl<ColorTy: ColorStruct + Pod> Pod for Alpha<ColorTy> {}

/// A premultiplied color with an alpha component.
///
/// The color components have been premultiplied by the alpha component.
#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct PremultipliedAlpha<ColorTy: ColorStruct> {
    /// The contained color, which has been premultiplied with `alpha`
    pub color: ColorTy,
    /// The alpha component.
    pub alpha: ColorTy::ComponentTy,
}

#[cfg(feature = "bytemuck")]
unsafe impl<ColorTy: ColorStruct + Zeroable> Zeroable for PremultipliedAlpha<ColorTy> {}
#[cfg(feature = "bytemuck")]
unsafe impl<ColorTy: ColorStruct + Pod> Pod for PremultipliedAlpha<ColorTy> {}

macro_rules! color_struct {
    {
        $(#[$doc:meta])*
        $name:ident<$default_component_ty:ty> {
            $($(#[$compdoc:meta])+
            $compname:ident,)+
        }
    } => {
        $(#[$doc])*
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
        pub struct $name<ComponentTy=$default_component_ty> {
            $($(#[$compdoc])+
            pub $compname: ComponentTy,)+
        }

        impl<CTy: Clone + Copy> ColorStruct for $name<CTy> {
            type ComponentTy = CTy;
        }

        #[cfg(feature = "bytemuck")]
        unsafe impl<ComponentTy: Zeroable> Zeroable for $name<ComponentTy> {}
        #[cfg(feature = "bytemuck")]
        unsafe impl<ComponentTy: Pod> Pod for $name<ComponentTy> {}

        impl<ComponentTy> From<[ComponentTy; 3]> for $name<ComponentTy> {
            fn from([$($compname),+]: [ComponentTy; 3]) -> $name<ComponentTy> {
                $name {
                    $($compname,)+
                }
            }
        }

        #[allow(clippy::from_over_into)]
        impl<ComponentTy> Into<[ComponentTy; 3]> for $name<ComponentTy> {
            fn into(self) -> [ComponentTy; 3] {
                let $name {
                    $($compname,)+
                } = self;
                [$($compname),+]
            }
        }

        impl<ComponentTy> AsRef<[ComponentTy; 3]> for $name<ComponentTy> {
            fn as_ref(&self) -> &[ComponentTy; 3] {
                unsafe { &*(self as *const $name<ComponentTy> as *const [ComponentTy; 3]) }
            }
        }

        macro_rules! impl_alpha_traits {
            ($alphaty:ident) => {
                impl<ComponentTy: Clone + Copy> From<$alphaty<$name<ComponentTy>>> for $name<ComponentTy> {
                    fn from(col_alpha: $alphaty<$name<ComponentTy>>) -> $name<ComponentTy> {
                        col_alpha.color
                    }
                }

                impl<ComponentTy: Clone + Copy> From<[ComponentTy; 4]> for $alphaty<$name<ComponentTy>> {
                    fn from([a, b, c, alpha]: [ComponentTy; 4]) -> $alphaty<$name<ComponentTy>> {
                        $alphaty {
                            color: $name::from([a, b, c]),
                            alpha
                        }
                    }
                }

                #[allow(clippy::from_over_into)]
                impl<ComponentTy: Clone + Copy> Into<[ComponentTy; 4]> for $alphaty<$name<ComponentTy>> {
                    fn into(self) -> [ComponentTy; 4] {
                        let $alphaty {
                            color,
                            alpha
                        } = self;

                        let $name {
                            $($compname,)+
                        } = color;

                        [$($compname,)+ alpha]
                    }
                }
            }
        }

        impl_alpha_traits!(Alpha);
        impl_alpha_traits!(PremultipliedAlpha);
    };
}

macro_rules! color_spaces {
    {
        $($(#[$space_doc:meta])*
        $space_name:ident<$default_component_ty:ty> {
            $($(#[$comp_doc:meta])+
            $comp_name:ident,)+
        })*
    } => {
        /// An enum with a variant for each of the color spaces
        /// supported by the library. Useful for tracking as metadata
        /// in something like an image type, and for runtime-determined color types.
        #[repr(u32)]
        pub enum Spaces {
            $(
                $(#[$space_doc])*
                $space_name,
            )*
        }

        $(
            color_struct! {
                $(#[$space_doc])*
                $space_name<$default_component_ty> {
                    $( $(#[$comp_doc])+
                    $comp_name,)+
                }
            }
        )*
    }
}

color_spaces! {
    /// A color in the encoded sRGB color space.
    ///
    /// This color space uses the sRGB/Rec.709 primaries, D65 white point,
    /// and sRGB transfer functions. The encoded version is nonlinear, with the
    /// sRGB OETF, aka "gamma compensation", applied.
    EncodedSrgb<u8> {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }

    /// A color in the linear (decoded) sRGB color space.
    ///
    /// This color space uses the sRGB/Rec.709 primaries, D65 white point,
    /// and sRGB transfer functions. This version is linear, with the
    /// sRGB EOTF, aka "inverse gamma compensation", applied in order to
    /// decode it from [`EncodedSrgb`]
    LinearSrgb<f32> {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }

    /// A color in the encoded Rec.709/BT.709 color space.
    ///
    /// This color space uses the BT.709 primaries, D65 white point,
    /// and BT.601 (reused in BT.709) transfer function. The encoded version is nonlinear, with the
    /// BT.601 OETF applied.
    EncodedRec709<u8> {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }

    /// A color in the Rec.709/BT.709 color space.
    ///
    /// This color space uses the BT.709 primaries, D65 white point,
    /// and BT.601 (reused in BT.709) transfer function. This version is linear, without the
    /// BT.601 OETF applied.
    Rec709<f32> {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }

    /// A color in a generic color space that can be represented by 3 components. The user
    /// is responsible for ensuring that the correct color space is respected.
    GenericColor<f32> {
        /// The first component.
        x,
        /// The second component.
        y,
        /// The third component.
        z,
    }

    /// A color in the ACEScg color space.
    ///
    /// This color space uses the ACES AP1 primaries and D60 white point.
    AcesCg<f32> {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }

    /// A color in the ACES 2065-1 color space.
    ///
    /// This color space uses the ACES AP0 primaries and D60 white point.
    Aces2065<f32> {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }

    /// A color in the ACEScc color space.
    ///
    /// This color space uses the ACES AP1 primaries and D60 white point
    /// and a pure logarithmic transfer function.
    AcesCc<f32> {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }

    /// A color in the ACEScct color space.
    ///
    /// This color space uses the ACES AP1 primaries and D60 white point
    /// and a logarithmic transfer function with a toe such that values
    /// are able to go negative.
    AcesCct<f32> {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }

    /// A color in the Display P3 (aka P3 D65) color space.
    ///
    /// This color space uses the P3 primaries and D65 white point
    /// and sRGB transfer functions. This version is linear,
    /// without the sRGB OETF applied.
    DisplayP3<f32> {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }

    /// A color in the Display P3 (aka P3 D65) color space.
    ///
    /// This color space uses the P3 primaries and D65 white point
    /// and sRGB transfer functions. This encoded version is nonlinear,
    /// with the sRGB OETF applied.
    EncodedDisplayP3<u8> {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }

    /// A color in the DCI-P3 (aka P3 DCI and P3 D60) color space.
    ///
    /// If you are looking for the P3 which is used on new Apple displays, see
    /// [`DisplayP3`] instead.
    ///
    /// This color space uses the P3 primaries and D60 white point.
    DciP3<f32> {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }

    /// A color in the X'Y'Z' color space, a DCI specification used for digital cinema mastering.
    ///
    /// This color space uses the CIE XYZ primaries, with special DCI white point and pure 2.6 gamma encoding.
    DciXYZPrime<f32> {
        /// The X' component.
        x,
        /// The Y' component.
        y,
        /// The Z' component.
        z,
    }

    /// A color in the BT.2020 color space.
    ///
    /// This color space uses the BT.2020 primaries and D65 white point.
    Bt2020<f32> {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }

    /// A color in the encoded BT.2020 color space.
    ///
    /// This color space uses the BT.2020 primaries and D65 white point and
    /// the BT.2020 transfer functions (equivalent to BT.601 transfer functions
    /// but with higher precision). This encoded version is nonlinear, with the
    /// BT.2020/BT.601 OETF applied.
    EncodedBt2020<f32> {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }

    /// A color in the BT.2100 color space.
    ///
    /// This color space uses the BT.2020 primaries and D65 white point.
    Bt2100<f32> {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }

    /// A color in the encoded BT.2100 color space with PQ (Perceptual Quantizer)
    /// transfer function.
    ///
    /// This color space uses the BT.2020 primaries and D65 white point and
    /// the ST 2084/"PQ" transfer function. It is nonlinear.
    EncodedBt2100PQ<f32> {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }

    /// A color in the encoded BT.2100 color space with HLG (Hybrid Log-Gamma)
    /// transfer function.
    ///
    /// This color space uses the BT.2020 primaries and D65 white point and
    /// the HLG transfer function. It is nonlinear.
    EncodedBt2100HLG<f32> {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }

    /// A color in the ICtCp color space with PQ (Perceptual Quantizer)
    /// nonlinearity.
    ///
    /// This color space is based on the BT.2020 primaries and D65 white point,
    /// but is not an RGB color space. Instead it is a roughly perceptual color
    /// space meant to more efficiently encode HDR content.
    ICtCpPQ<f32> {
        /// The I (intensity) component.
        i,
        /// The Ct (chroma-tritan) component.
        ct,
        /// The Cp (chroma-protan) component.
        cp,
    }

    /// A color in the ICtCp color space with HLG (Hybrid Log-Gamma)
    /// nonlinearity.
    ///
    /// This color space is based on the BT.2020 primaries and D65 white point,
    /// but is not an RGB color space. Instead it is a roughly perceptual color
    /// space meant to more efficiently encode HDR content.
    ICtCpHLG<f32> {
        /// The I (intensity) component.
        i,
        /// The Ct (chroma-tritan) component.
        ct,
        /// The Cp (chroma-protan) component.
        cp,
    }

    /// A color in the CIE XYZ color space.
    ///
    /// This color space uses the CIE XYZ primaries and D65 white point.
    CieXYZ<f32> {
        /// The X component.
        x,
        /// The Y component.
        y,
        /// The Z component.
        z,
    }

    /// A color in the CIE L\*a\*b\* color space.
    CieLab<f32> {
        /// The L (lightness) component. Varies from 0 to 100.
        l,
        /// The a component, representing green-red chroma difference.
        a,
        /// The b component, representing blue-yellow chroma difference.
        b,
    }

    /// A color in the CIE L\*C\*h° color space.
    CieLCh<f32> {
        /// The L (lightness) component. Varies from 0 to 100.
        l,
        /// The C (chroma) component. Varies from 0 to a hue dependent maximum.
        c,
        /// The h (hue) component. Varies from -PI to PI.
        h,
    }

    /// A color in the Oklab color space.
    Oklab<f32> {
        /// The L (lightness) component. Varies from 0 to 1
        l,
        /// The a component, representing green-red chroma difference.
        a,
        /// The b component, representing blue-yellow chroma difference.
        b,
    }

    /// A color in the Oklch color space (a transformation from Oklab to LCh° coordinates).
    Oklch<f32> {
        /// The L (lightness) component. Varies from 0 to 1.
        l,
        /// The C (chroma) component. Varies from 0 to a hue dependent maximum.
        c,
        /// The h (hue) component. Varies from -PI to PI.
        h,
    }

    /// A color in the HSL color space.
    ///
    /// Since HSL is a relative color space, it is required to know the RGB space which
    /// it was transformed from. We define this as the linear sRGB space, as that is
    /// the most common case.
    Hsl<f32> {
        /// The H (hue) component. Varies from 0 to 1.
        h,
        /// The S (saturation) component. Varies from 0 to 1.
        s,
        /// The L (lightness) component. Varies from 0 to 1.
        l,
    }

    /// A color in the HSV color space.
    ///
    /// Since HSV is a relative color space, it is required to know the RGB space which
    /// it was transformed from. We define this as the linear sRGB space, as that is
    /// the most common case.
    Hsv<f32> {
        /// The H (hue) component. Varies from 0 to 1.
        h,
        /// The S (saturation) component. Varies from 0 to 1.
        s,
        /// The V (value) component. Varies from 0 to 1.
        v,
    }

    /// A color in the YCbCr color space. See discussion of the difference between YCbCr, YUV, and
    /// YPbPr in [YCbCr Wikipedia article](https://en.wikipedia.org/wiki/YCbCr)
    ///
    /// Since YCbCr is a relative color space, it is required to know the RGB space which
    /// it was transformed from. A common base color space for YCbCr is 
    YCbCr<u8> {
        /// The Y (luminance) component.
        y,
        /// The Cb (chroma-blue/yellow) component.
        cb,
        /// The Cr (chroma-red/green) component.
        cr,
    }

    /// A color in the YPbPr color space. See discussion of the difference between YCbCr, YUV, and
    /// YPbPr in [YCbCr Wikipedia article](https://en.wikipedia.org/wiki/YCbCr)
    YPbPr<f32> {
        /// The Y (luminance) component.
        y,
        /// The Pb (chroma-blue/yellow) component.
        pb,
        /// The Pr (chroma-red/green) component.
        pr,
    }

    /// A color in the YUV color space. See discussion of the difference between YCbCr, YUV, and
    /// YPbPr in [YCbCr Wikipedia article](https://en.wikipedia.org/wiki/YCbCr)
    Yuv<f32> {
        /// The Y (luminance) component.
        y,
        /// The U (chroma-blue/yellow) component.
        u,
        /// The V (chroma-red/green) component.
        v,
    }

    /// A color in the YCxCz (also called YyCxCz) color space, originally defined in "Optimized
    /// universal color palette design for error diffusion" by B. W. Kolpatzik and C. A. Bouman. 
    YCxCz<f32> {
        /// The Yy (luminance) component.
        y,
        /// The Cx (chroma difference blue/yellow) component
        cx,
        /// The Cz (chroma difference red/green) component
        cz,
    }
}
