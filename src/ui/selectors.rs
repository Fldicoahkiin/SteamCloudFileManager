// 全选所有文件
pub fn select_all_files(file_count: usize) -> Vec<usize> {
    (0..file_count).collect()
}

// 反选文件
pub fn invert_file_selection(current_selected: &[usize], file_count: usize) -> Vec<usize> {
    let current_set: std::collections::HashSet<_> = current_selected.iter().copied().collect();
    (0..file_count)
        .filter(|i| !current_set.contains(i))
        .collect()
}

// 清除文件选择
pub fn clear_file_selection() -> Vec<usize> {
    Vec::new()
}
