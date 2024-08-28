mod nat256 {
    use crate::Nat256;
    use candid::{Decode, Encode, Nat};
    use num_bigint::BigUint;
    use proptest::{arbitrary::any, prelude::Strategy, proptest};

    proptest! {
        #[test]
        fn should_encode_decode(u256 in arb_u256()) {
            encode_decode_roundtrip(u256);
        }

        #[test]
        fn should_fail_to_decode_nat_overflowing_a_u256(offset in any::<u64>()) {
            let u256_max: BigUint = BigUint::from_bytes_be(&[0xff; 32]);
            encode_decode_roundtrip(u256_max.clone());

            let offset = BigUint::from(offset);
            let overflow_u256 = Nat::from(u256_max + offset);
            let encoded_overflow_u256 = Encode!(&overflow_u256).unwrap();

            let decoded_overflow_nat256: Result<Nat256, _> = Decode!(&encoded_overflow_u256, Nat256);
            let error_msg = format!("{:?}", decoded_overflow_nat256.unwrap_err());

            assert!(
                error_msg.contains("Deserialize error: Nat does not fit in a U256"),
                "Unexpected error message: {}",
                error_msg
            );
        }

        #[test]
        fn should_convert_to_bytes_and_back(u256 in arb_u256()) {
            let value = Nat256::try_from(Nat::from(u256)).unwrap();
            let bytes = value.clone().into_be_bytes();

            let value_from_bytes = Nat256::from_be_bytes(bytes);

            assert_eq!(value, value_from_bytes);
        }
    }

    fn encode_decode_roundtrip(value: BigUint) {
        let nat = Nat::from(value);
        let encoded_nat = Encode!(&nat).unwrap();

        let nat256 = Nat256::try_from(nat.clone()).unwrap();
        let encoded_nat256 = Encode!(&nat256).unwrap();

        assert_eq!(encoded_nat, encoded_nat256);

        let decoded_nat256: Nat256 = Decode!(&encoded_nat, Nat256).unwrap();
        assert_eq!(decoded_nat256.0, nat);
    }

    fn arb_u256() -> impl Strategy<Value = BigUint> {
        use proptest::array::uniform32;
        uniform32(any::<u8>()).prop_map(|value| BigUint::from_bytes_be(&value))
    }
}

mod hex_string {
    use crate::{Hex, Hex20, Hex32};
    use candid::{CandidType, Decode, Encode};
    use proptest::prelude::{Strategy, TestCaseError};
    use proptest::{prop_assert, prop_assert_eq, proptest};
    use serde::de::DeserializeOwned;
    use std::ops::RangeInclusive;
    use std::str::FromStr;

    proptest! {
        #[test]
        fn should_encode_decode(
            hex20 in arb_var_len_hex_string(20..=20_usize),
            hex32 in arb_var_len_hex_string(32..=32_usize),
            hex in arb_var_len_hex_string(0..=100_usize)
        ) {
            encode_decode_roundtrip::<Hex20>(&hex20)?;
            encode_decode_roundtrip::<Hex32>(&hex32)?;
            encode_decode_roundtrip::<Hex>(&hex)?;
        }

        #[test]
        fn should_fail_to_decode_strings_with_wrong_length(
            short_hex20 in arb_var_len_hex_string(0..=19_usize),
            long_hex20 in arb_var_len_hex_string(21..=100_usize),
            short_hex32 in arb_var_len_hex_string(0..=31_usize),
            long_hex32 in arb_var_len_hex_string(33..=100_usize),
        ) {
            let decoded_short_hex20 = Decode!(&Encode!(&short_hex20).unwrap(), Hex20);
            let decoded_long_hex20 = Decode!(&Encode!(&long_hex20).unwrap(), Hex20);
            for result in [decoded_short_hex20, decoded_long_hex20] {
                prop_assert!(
                    result.is_err(),
                    "Expected error decoding hex20 with wrong length, got: {:?}",
                    result
                );
            }

            let decoded_short_hex32 = Decode!(&Encode!(&short_hex32).unwrap(), Hex32);
            let decoded_long_hex32 = Decode!(&Encode!(&long_hex32).unwrap(), Hex32);
            for result in [decoded_short_hex32, decoded_long_hex32] {
                prop_assert!(
                    result.is_err(),
                    "Expected error decoding hex32 with wrong length, got: {:?}",
                    result
                );
            }
        }
    }

    fn encode_decode_roundtrip<T>(value: &str) -> Result<(), TestCaseError>
    where
        T: FromStr + CandidType + DeserializeOwned + PartialEq + std::fmt::Debug,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let hex: T = value.parse().unwrap();

        let encoded_text_value = Encode!(&value.to_lowercase()).unwrap();
        let encoded_hex = Encode!(&hex).unwrap();
        prop_assert_eq!(
            &encoded_text_value,
            &encoded_hex,
            "Encode value differ for {}",
            value
        );

        let decoded_hex = Decode!(&encoded_text_value, T).unwrap();
        prop_assert_eq!(&decoded_hex, &hex, "Decode value differ for {}", value);
        Ok(())
    }

    fn arb_var_len_hex_string(
        num_bytes_range: RangeInclusive<usize>,
    ) -> impl Strategy<Value = String> {
        num_bytes_range.prop_flat_map(|num_bytes| {
            proptest::string::string_regex(&format!("0x[0-9a-fA-F]{{{}}}", 2 * num_bytes)).unwrap()
        })
    }
}
