pub mod board;

mod tests {
    use crate::board::Board;
    
    #[test]
    fn coordinate_to_bit_test() {
        let mask = Board::coordinate_to_bit((0, 0));

        assert_eq!(
            0b_10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
            mask
        );
    
        let mask = Board::coordinate_to_bit((5, 4));
        assert_eq!(
            0b_00000000_00000000_00000000_00000000_00000000_00001000_00000000_00000000,
            mask
        );
    }
}
