#[cfg(test)]
mod tests {
    use crate::build_utf16_index_map;

    #[test]
    fn test_utf16_index_map_ascii() {
        let source = "import 'bar';";
        let map = build_utf16_index_map(source);
        
        // ASCII: each byte = 1 UTF-16 code unit
        assert_eq!(map.len(), source.len() + 1);
        assert_eq!(map[0], 0);
        assert_eq!(map[8], 8); // Position of 'b' in 'bar'
        assert_eq!(map[11], 11); // Position after 'r'
    }

    #[test]
    fn test_utf16_index_map_emoji() {
        let source = "import '😀';";
        let map = build_utf16_index_map(source);
        
        // '😀' is 4 bytes in UTF-8, 2 code units in UTF-16
        println!("Source: {:?}", source);
        println!("Source bytes: {:?}", source.as_bytes());
        println!("Map length: {}", map.len());
        println!("Map: {:?}", map);
        
        assert_eq!(map[0], 0);
        assert_eq!(map[8], 8); // Position before emoji
        // Emoji starts at byte 8, is 4 bytes long
        // In UTF-16, it's 2 code units
        assert_eq!(map[12], 10); // Position after emoji (8 + 2)
    }

    #[test]
    fn test_utf16_index_map_chinese() {
        let source = "import '你好';";
        let map = build_utf16_index_map(source);
        
        // '你' and '好' are each 3 bytes in UTF-8, 1 code unit in UTF-16
        println!("Source: {:?}", source);
        println!("Source bytes: {:?}", source.as_bytes());
        println!("Map length: {}", map.len());
        println!("Map: {:?}", map);
        
        assert_eq!(map[0], 0);
        assert_eq!(map[8], 8); // Position before '你'
        assert_eq!(map[11], 9); // Position after '你' (8 + 1)
        assert_eq!(map[14], 10); // Position after '好' (9 + 1)
    }
}
