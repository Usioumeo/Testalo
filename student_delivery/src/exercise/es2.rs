/*!
Create a struct TreeNode generic over T that represents a binary tree.
It should have a field value of type T and two optional fields left and right (they
should hold a pointer to another TreeNode ).
Implement:
a method new that takes a value and returns a new TreeNode with the given value
and no children.
a method from_vec that takes a vector of values and returns a TreeNode with the
given values.
a method insert that takes a value and inserts it into the tree (follow binary search
tree rules).
*/

type TreePointer<T> = Box<TreeNode<T>>;



#[derive(PartialOrd, Clone, PartialEq)]
struct TreeNode<T: PartialOrd+ PartialEq + Clone>{
    inner: T,
    left: Option<TreePointer<T>>,
    right: Option<TreePointer<T>>,
}

impl<T: PartialOrd + PartialEq+ Clone> TreeNode<T>{
    fn new(inner: T)->Self{
        Self{
            left: None,
            right: None,
            inner,
        }
    }
    fn from_vec(v: Vec<T>)->Self{
        let mut iter = v.into_iter();
        let mut ret = Self::new(iter.next().unwrap());
        while let Some(x) = iter.next(){
            ret.insert(x);
        }
        ret
    }
    fn insert(&mut self, inner: T){
        if self.inner>inner{
            if let Some(x) = &mut self.left{
                x.insert(inner);
            }else{
                self.left=Some(Box::new(TreeNode::new(inner)));
            }
        }else{
            if let Some(x) = &mut self.right{
                x.insert(inner);
            }else{
                self.right=Some(Box::new(TreeNode::new(inner)));
            }
        }
    }
}
#[runtest]
#[refers_to(TreeNode<T>::new)]
/// check if the related struct exists
fn struct_exist(){
    let t = TreeNode::new(10);
    let s = TreeNode::new("String".to_string());
}

/// does new works?
#[runtest]
fn test_new(){
    let t = TreeNode::new(10);
    let s = TreeNode::new("String".to_string());
}
#[runtest]
#[refers_to(TreeNode<T>::new)]
/// check if it is possible to add a value
fn insert_value(){
    let mut t = TreeNode::new(5);
    t.insert(6);
    t.insert(4);
}

#[runtest]
#[refers_to(TreeNode<T>::new)]
/// from Vec test
fn insert_value(){
    
}