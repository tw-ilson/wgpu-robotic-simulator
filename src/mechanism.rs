use crate::urdf::*;
use crate::geometry::*;
type ChainError = Box<dyn std::error::Error>;

// struct SerialChainMechanism {
//     tlist: Vec<Transform>,
// }

trait Chain {
    fn check(self) -> Result<Self, ChainError> where Self:Sized;
    fn make_tlist(&self) -> Vec<Transform>;
    fn chain_forward_kinematics(&self) -> Result<glm::Vec3, ChainError>;
    fn chain_inverse_kinematics(&self, posn: glm::Vec3) -> Result<Vec<f32>, ChainError>;
}

impl Chain for RobotDescriptor {
    fn check(self) -> Result<Self, ChainError> {
        let mut cur_link = self.links.get(0).expect("No links found!");
        loop {
            let mut jlist = self.joints.iter().filter(|j| self.links[j.parent].link_name == cur_link.link_name);
            if let Some(joint) = jlist.next() {
                if !jlist.next().is_none() { return Err("link has multiple child links -- chain invalid".into())}
                cur_link = &self.links[joint.child];
            } else { break };
        }
        return Ok(self);
    }
    // End-effector is defined as origin of last link of chain
    fn chain_forward_kinematics(&self) -> Result<glm::Vec3, ChainError> {
        
    }
    fn chain_inverse_kinematics(&self, posn: glm::Vec3) -> Result<Vec<f32>, ChainError> {
        unimplemented!()
    }
}

