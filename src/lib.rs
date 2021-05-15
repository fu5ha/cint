//! This library is a lean, minimal, and stable set of types
//! for color interoperation between crates in Rust. Its goal is to serve the same
//! function that [`mint`](https://docs.rs/mint/) provides for (linear algebra) math types.
//! It does not actually provide any conversion, math, etc. for these types, but rather
//! serves as a stable interface that multiple libraries can rely on and then convert
//! to their own internal representations to actually use. It is also `#![no_std]`.
//! [`bytemuck`][https://docs.rs/bytemuck/] impls are provided with the `bytemuck` feature.
//!
//! # How to Use
//!
//! If you have no idea about color management or encoding principles but you want to
//! use this crate in your own, here's a *very basic* rundown.
//!
//! If you have a color that you loaded from an 8-bit format like a PNG, JPG, etc.,
//! **or** if you have a color that you picked from some sort of online color picker
//! or in Photoshop or Aseprite, then what you have is almost certainly an [`EncodedSrgb<u8>`]
//! color (or if it has an alpha channel, [`EncodedSrgbA<u8>`]). If you have a color that you loaded
//! from a similar format but has floating point values instead of `u8` ints, then you
//! almost certainly instead have a [`EncodedSrgb<f32>`] color (or [`EncodedSrgbA<f32>`] if it
//! has an alpha channel).
//!
//! If you "linearized" or performed "inverse gamma correction" on such a color, then you instead
//! might have a [`LinearSrgb<f32>`]/[`LinearSrgbA<f32>`].
//!
//! If you are more familiar with color encoding, then you'll find a collection of other color spaces
//! represented, as well as the generic [`GenericColor<T>`]/[`GenericColorAlpha<T>`] types which
//! can be used if the color space you wish to use is not represented.
//!
//! ## Colors with alpha channels
//!
//! Note that colors with alpha channels are paramaterized not only by their `ComponentTy` (i.e.
//! `f32`, `u8`, etc.), but also by an `AlphaState`. This should be one of [`Premultiplied`]
//! or [`Separate`] and
#![no_std]

#[cfg(feature = "bytemuck")]
use bytemuck::{Pod, Zeroable};

macro_rules! color_struct {
    {
        $(#[$doc:meta])*
        $name:ident {
            $($(#[$compdoc:meta])+
            $compname:ident,)+
        }
    } => {
        $(#[$doc])*
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
        pub struct $name<ComponentTy> {
            $($(#[$compdoc])+
            pub $compname: ComponentTy,)+
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
    };
    {
        $(#[$doc:meta])*
        $name:ident, $alphaname:ident {
            $($(#[$compdoc:meta])+
            $compname:ident,)+
        }
    } => {
        color_struct! {
            $(#[$doc])*
            $name {
                $($(#[$compdoc])+
                $compname,)+
            }
        }

        $(#[$doc])*
        ///
        /// This is type has an alpha channel, and so is paramaterized by both
        /// a `ComponentTy`, as with any color type, but also an `AlphaState`,
        /// which symbolizes whether the other color components have been
        /// [`Premultiplied`] with the alpha channel or are [`Separate`] from it.
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
        pub struct $alphaname<ComponentTy, AlphaState> {
            $($(#[$compdoc])+
            pub $compname: ComponentTy,)+
            /// The alpha component.
            pub alpha: ComponentTy,
            _pd: core::marker::PhantomData<AlphaState>,
        }

        #[cfg(feature = "bytemuck")]
        unsafe impl<ComponentTy: Zeroable, AlphaState> Zeroable for $alphaname<ComponentTy, AlphaState> {}
        #[cfg(feature = "bytemuck")]
        unsafe impl<ComponentTy: Pod, AlphaState: Copy + 'static> Pod for $alphaname<ComponentTy, AlphaState> {}

        #[allow(clippy::from_over_into)]
        impl<ComponentTy, AlphaState> Into<[ComponentTy; 4]> for $alphaname<ComponentTy, AlphaState> {
            fn into(self) -> [ComponentTy; 4] {
                let $alphaname {
                    $($compname,)+
                    alpha,
                    ..
                } = self;
                [$($compname,)+ alpha]
            }
        }

        impl<ComponentTy, AlphaState> AsRef<[ComponentTy; 4]> for $alphaname<ComponentTy, AlphaState> {
            fn as_ref(&self) -> &[ComponentTy; 4] {
                unsafe { &*(self as *const $alphaname<ComponentTy, AlphaState> as *const [ComponentTy; 4]) }
            }
        }
    };
}

color_struct! {
    /// A color in the encoded sRGB color space.
    ///
    /// This color space uses the sRGB/Rec.709 primaries, D65 white point,
    /// and sRGB transfer functions. The encoded version is nonlinear, with the
    /// sRGB OETF, aka "gamma compensation", applied.
    EncodedSrgb, EncodedSrgbA {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }
}

color_struct! {
    /// A color in the linear (decoded) sRGB color space.
    ///
    /// This color space uses the sRGB/Rec.709 primaries, D65 white point,
    /// and sRGB transfer functions. This version is linear, with the
    /// sRGB EOTF, aka "inverse gamma compensation", applied in order to
    /// decode it from [`EncodedSrgb`]
    LinearSrgb, LinearSrgbA {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }
}

color_struct! {
    /// A color in the encoded Rec.709/BT.709 color space.
    ///
    /// This color space uses the BT.709 primaries, D65 white point,
    /// and BT.601 (reused in BT.709) transfer function. The encoded version is nonlinear, with the
    /// BT.601 OETF applied.
    EncodedRec709 {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }
}

color_struct! {
    /// A color in the Rec.709/BT.709 color space.
    ///
    /// This color space uses the BT.709 primaries, D65 white point,
    /// and BT.601 (reused in BT.709) transfer function. This version is linear, without the
    /// BT.601 OETF applied.
    Rec709 {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }
}

color_struct! {
    /// A color in a generic color space that can be represented by 3 components. The user
    /// is responsible for ensuring that the correct color space is respected.
    GenericColor, GenericColorAlpha {
        /// The first component.
        comp1,
        /// The second component.
        comp2,
        /// The third component.
        comp3,
    }
}

color_struct! {
    /// A color in the ACEScg color space.
    ///
    /// This color space uses the ACES AP1 primaries and D60 white point.
    AcesCg {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }
}

color_struct! {
    /// A color in the ACES 2065-1 color space.
    ///
    /// This color space uses the ACES AP0 primaries and D60 white point.
    Aces2065 {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }
}

color_struct! {
    /// A color in the ACEScc color space.
    ///
    /// This color space uses the ACES AP1 primaries and D60 white point
    /// and a pure logarithmic transfer function.
    AcesCc {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }
}

color_struct! {
    /// A color in the ACEScct color space.
    ///
    /// This color space uses the ACES AP1 primaries and D60 white point
    /// and a logarithmic transfer function with a toe such that values
    /// are able to go negative.
    AcesCct {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }
}

color_struct! {
    /// A color in the Display P3 (aka P3 D65) color space.
    ///
    /// This color space uses the P3 primaries and D65 white point
    /// and sRGB transfer functions. This version is linear,
    /// without the sRGB OETF applied.
    DisplayP3, DisplayP3A {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }
}

color_struct! {
    /// A color in the Display P3 (aka P3 D65) color space.
    ///
    /// This color space uses the P3 primaries and D65 white point
    /// and sRGB transfer functions. This encoded version is nonlinear,
    /// with the sRGB OETF applied.
    EncodedDisplayP3, EncodedDisplayP3A {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }
}

color_struct! {
    /// A color in the DCI-P3 (aka P3 DCI and P3 D60) color space.
    ///
    /// If you are looking for the P3 which is used on new Apple displays, see
    /// [`DisplayP3`] instead.
    ///
    /// This color space uses the P3 primaries and D60 white point.
    DciP3 {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }
}

color_struct! {
    /// A color in the X'Y'Z' color space, a DCI specification used for digital cinema mastering.
    ///
    /// This color space uses the CIE XYZ primaries, with special DCI white point and pure 2.6 gamma encoding.
    DciXYZPrime {
        /// The X' component.
        x,
        /// The Y' component.
        y,
        /// The Z' component.
        z,
    }
}

color_struct! {
    /// A color in the BT.2020 color space.
    ///
    /// This color space uses the BT.2020 primaries and D65 white point.
    Bt2020 {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }
}

color_struct! {
    /// A color in the encoded BT.2020 color space.
    ///
    /// This color space uses the BT.2020 primaries and D65 white point and
    /// the BT.2020 transfer functions (equivalent to BT.601 transfer functions
    /// but with higher precision). This encoded version is nonlinear, with the
    /// BT.2020/BT.601 OETF applied.
    EncodedBt2020 {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }
}

color_struct! {
    /// A color in the BT.2100 color space.
    ///
    /// This color space uses the BT.2020 primaries and D65 white point.
    Bt2100 {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }
}

color_struct! {
    /// A color in the encoded BT.2100 color space with PQ (Perceptual Quantizer)
    /// transfer function.
    ///
    /// This color space uses the BT.2020 primaries and D65 white point and
    /// the ST 2084/"PQ" transfer function. It is nonlinear.
    EncodedBt2100PQ {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }
}

color_struct! {
    /// A color in the encoded BT.2100 color space with HLG (Hybrid Log-Gamma)
    /// transfer function.
    ///
    /// This color space uses the BT.2020 primaries and D65 white point and
    /// the HLG transfer function. It is nonlinear.
    EncodedBt2100HLG {
        /// The red component.
        r,
        /// The green component.
        g,
        /// The blue component.
        b,
    }
}

color_struct! {
    /// A color in the ICtCp color space with PQ (Perceptual Quantizer)
    /// nonlinearity.
    ///
    /// This color space is based on the BT.2020 primaries and D65 white point,
    /// but is not an RGB color space. Instead it is a roughly perceptual color
    /// space meant to more efficiently encode HDR content.
    ICtCpPQ {
        /// The I (intensity) component.
        i,
        /// The Ct (chroma-tritan) component.
        ct,
        /// The Cp (chroma-protan) component.
        cp,
    }
}

color_struct! {
    /// A color in the ICtCp color space with HLG (Hybrid Log-Gamma)
    /// nonlinearity.
    ///
    /// This color space is based on the BT.2020 primaries and D65 white point,
    /// but is not an RGB color space. Instead it is a roughly perceptual color
    /// space meant to more efficiently encode HDR content.
    ICtCpHLG {
        /// The I (intensity) component.
        i,
        /// The Ct (chroma-tritan) component.
        ct,
        /// The Cp (chroma-protan) component.
        cp,
    }
}

color_struct! {
    /// A color in the CIE XYZ color space.
    ///
    /// This color space uses the CIE XYZ primaries and D65 white point.
    CieXYZ, CieXYZA {
        /// The X component.
        x,
        /// The Y component.
        y,
        /// The Z component.
        z,
    }
}

color_struct! {
    /// A color in the CIE L\*a\*b color space.
    CieLab, CieLabA {
        /// The L (lightness) component. Varies from 0 to 100.
        l,
        /// The a component, representing green-red chroma difference.
        a,
        /// The b component, representing blue-yellow chroma difference.
        b,
    }
}

color_struct! {
    /// A color in the CIE L\*C\*h color space.
    CieLCh, CieLChA {
        /// The L (lightness) component. Varies from 0 to 100.
        l,
        /// The C (chroma) component. Varies from 0 to a hue dependent maximum.
        c,
        /// The h (hue) component. Varies from -PI to PI.
        h,
    }
}

color_struct! {
    /// A color in the Oklab color space.
    Oklab, OklabA {
        /// The L (lightness) component. Varies from 0 to 1
        l,
        /// The a component, representing green-red chroma difference.
        a,
        /// The b component, representing blue-yellow chroma difference.
        b,
    }
}

color_struct! {
    /// A color in the Oklch color space (a transformation from Oklab to L\*c\*h coordinates).
    Oklch, OklchA {
        /// The L (lightness) component. Varies from 0 to 1.
        l,
        /// The C (chroma) component. Varies from 0 to a hue dependent maximum.
        c,
        /// The h (hue) component. Varies from -PI to PI.
        h,
    }
}

/// Symbolizes that a color's component values have been premultiplied with its alpha channel.
///
/// Used to paramaterize colors with alpha channels alongside [Separate].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Premultiplied {}

/// Symbolizes that a color's component values are separate from its alpha channel.
///
/// Used to paramaterize colors with alpha channels alongside [Premultiplied].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Separate {}
