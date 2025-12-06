use crate::steam_api::CloudFile;

#[derive(Debug, Clone)]
pub enum FileTreeNode {
    Folder {
        name: String,
        path: String,
        children: Vec<FileTreeNode>,
        is_expanded: bool,
        file_count: usize,
        total_size: u64,
        root_description: String,
    },
    File {
        name: String,
        index: usize,
        file: CloudFile,
    },
}

impl FileTreeNode {
    pub fn name(&self) -> &str {
        match self {
            FileTreeNode::Folder { name, .. } => name,
            FileTreeNode::File { name, .. } => name,
        }
    }

    pub fn is_folder(&self) -> bool {
        matches!(self, FileTreeNode::Folder { .. })
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<FileTreeNode>> {
        match self {
            FileTreeNode::Folder { children, .. } => Some(children),
            FileTreeNode::File { .. } => None,
        }
    }
}

// 文件树管理器
pub struct FileTree {
    root: FileTreeNode,
}

impl FileTree {
    pub fn new(files: &[CloudFile]) -> Self {
        let root = build_tree(files);
        Self { root }
    }

    pub fn root_mut(&mut self) -> &mut FileTreeNode {
        &mut self.root
    }
}

// 解析文件路径，返回文件夹路径部分
// 最后一个 `/` 后面是文件名，前面是路径
fn parse_file_path(file: &CloudFile) -> String {
    if file.name.contains('/') {
        if let Some(last_slash) = file.name.rfind('/') {
            file.name[..last_slash].to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

// 获取文件名
fn get_file_name(file: &CloudFile) -> String {
    if let Some(last_slash) = file.name.rfind('/') {
        file.name[last_slash + 1..].to_string()
    } else {
        file.name.clone()
    }
}

// 构建文件树
fn build_tree(files: &[CloudFile]) -> FileTreeNode {
    let mut root = FileTreeNode::Folder {
        name: "Root".to_string(),
        path: String::new(),
        children: Vec::new(),
        is_expanded: true,
        file_count: 0,
        total_size: 0,
        root_description: String::new(),
    };

    for (index, file) in files.iter().enumerate() {
        let path = parse_file_path(file);
        let filename = get_file_name(file);

        if path.is_empty() {
            // 根目录文件
            if let FileTreeNode::Folder { children, .. } = &mut root {
                children.push(FileTreeNode::File {
                    name: filename,
                    index,
                    file: file.clone(),
                });
            }
        } else {
            // 有路径的文件
            let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            insert_into_tree(
                &mut root,
                &parts,
                filename,
                file.name.clone(),
                index,
                file.clone(),
                file.root_description.clone(),
            );
        }
    }

    // 排序：文件夹优先
    sort_children(&mut root);

    // 更新文件夹统计信息
    update_folder_stats(&mut root);

    root
}

// 递归插入文件到树中
fn insert_into_tree(
    node: &mut FileTreeNode,
    parts: &[&str],
    filename: String,
    _full_path: String,
    index: usize,
    file: CloudFile,
    root_description: String,
) {
    if parts.is_empty() {
        // 到达目标位置，插入文件
        if let FileTreeNode::Folder { children, .. } = node {
            children.push(FileTreeNode::File {
                name: filename,
                index,
                file,
            });
        }
        return;
    }

    let current_part = parts[0];
    let remaining_parts = &parts[1..];

    if let FileTreeNode::Folder { children, path, .. } = node {
        // 构建子文件夹的完整路径
        let child_path = if path.is_empty() {
            current_part.to_string()
        } else {
            format!("{}/{}", path, current_part)
        };

        // 查找是否已存在该文件夹
        let child = children
            .iter_mut()
            .find(|c| c.is_folder() && c.name() == current_part);

        if let Some(child) = child {
            // 文件夹已存在，继续递归
            insert_into_tree(
                child,
                remaining_parts,
                filename,
                _full_path,
                index,
                file,
                root_description,
            );
        } else {
            // 创建新文件夹
            let mut new_folder = FileTreeNode::Folder {
                name: current_part.to_string(),
                path: child_path,
                children: Vec::new(),
                is_expanded: true,
                file_count: 0,
                total_size: 0,
                root_description: root_description.clone(),
            };
            insert_into_tree(
                &mut new_folder,
                remaining_parts,
                filename,
                _full_path,
                index,
                file,
                root_description,
            );
            children.push(new_folder);
        }
    }
}

// 排序子节点：文件夹优先，然后按名称排序
fn sort_children(node: &mut FileTreeNode) {
    if let FileTreeNode::Folder { children, .. } = node {
        children.sort_by(|a, b| match (a.is_folder(), b.is_folder()) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name().cmp(b.name()),
        });

        // 递归排序子文件夹
        for child in children {
            sort_children(child);
        }
    }
}

// 更新文件夹统计信息（文件数量和总大小）
fn update_folder_stats(node: &mut FileTreeNode) -> (usize, u64) {
    match node {
        FileTreeNode::Folder {
            children,
            file_count,
            total_size,
            ..
        } => {
            let mut count = 0;
            let mut size = 0;

            for child in children {
                let (child_count, child_size) = update_folder_stats(child);
                count += child_count;
                size += child_size;
            }

            *file_count = count;
            *total_size = size;
            (count, size)
        }
        FileTreeNode::File { .. } => (1, 0),
    }
}
