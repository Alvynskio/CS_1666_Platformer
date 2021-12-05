use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use crate::player::Player;
use crate::object_controller::ObjectController;

const PACKET_SIZE: usize = 52;
const DEBUG: bool = false;

#[derive(Copy, Clone)]
pub enum Mode {
    MultiplayerPlayer1,
    MultiplayerPlayer2,
}

pub struct Network {
    socket: UdpSocket,
    pub mode: Mode,
}

impl Network {
    pub fn new(mode: Mode) -> Network {
        let new_network = |local, remote| {
            let socket = UdpSocket::bind(local).expect("couldn't bind to local");
            socket.connect(remote).expect("couldn't connect to remote");
            Network { socket, mode }
        };
        match mode {
            Mode::MultiplayerPlayer1 => {
                let local = SocketAddr::from_str("127.0.0.1:34254").unwrap();
                let remote = SocketAddr::from_str("127.0.0.1:34255").unwrap();
                new_network(local, remote)
            }
            Mode::MultiplayerPlayer2 => {
                let local = SocketAddr::from_str("127.0.0.1:34255").unwrap();
                let remote = SocketAddr::from_str("127.0.0.1:34254").unwrap();
                new_network(local, remote)
            }
        }
    }

    pub fn get_network_mode(&self) -> Mode {
        self.mode
    }

    pub fn get_packet_buffer(&self) -> [u8; PACKET_SIZE] {
        let mut buf: [u8; PACKET_SIZE] = [0; PACKET_SIZE];
        let (_amt, _src) =  self.socket.recv_from(& mut buf).unwrap();
        return buf;
    }

    pub fn pack_and_send_data(
        &self, player: &mut Player,
        block: &ObjectController,
    ) -> std::io::Result<usize> {

        //Player Information
        let player_xpos = player.physics.x().to_le_bytes();
        let player_ypos = player.physics.y().to_le_bytes();
        let flip = player.flip_horizontal as u32;
        let flip = flip.to_le_bytes();
        let anim = player.anim.next_anim();
        let ax = anim.x().to_le_bytes();
        let ay = match self.mode {
            Mode::MultiplayerPlayer1 => anim.y(),
            Mode::MultiplayerPlayer2 => (2*anim.height()) as i32 + anim.y(),
        }.to_le_bytes(); // to get to green characters
        let aw = anim.width().to_le_bytes();
        let ah = anim.height().to_le_bytes();

        //Portal Information
        let portal_x: [u8; 4];
        let portal_y: [u8; 4];
        let portal_rotation: [u8; 4];

        match self.mode {
            Mode::MultiplayerPlayer1 => {
                portal_x = player.portal.portals[0].x().to_le_bytes();
                portal_y = player.portal.portals[0].y().to_le_bytes();
                portal_rotation = player.portal.portals[0].rotation().to_le_bytes();
            },
            Mode::MultiplayerPlayer2 => {
                portal_x = player.portal.portals[1].x().to_le_bytes();
                portal_y = player.portal.portals[1].y().to_le_bytes();
                portal_rotation = player.portal.portals[1].rotation().to_le_bytes();
            }
        }

        //Block Information
        let block_x: [u8; 4] = block.x().to_le_bytes();
        let block_y: [u8; 4] = block.y().to_le_bytes();
        let carried = block.carried as u32;
        let block_carried: [u8; 4] = carried.to_le_bytes();

        let buf = [
            player_xpos,
            player_ypos,
            flip,
            portal_x,
            portal_y,
            portal_rotation,
            ax,
            ay,
            aw,
            ah,
            block_x,
            block_y,
            block_carried,
        ].concat();
        if DEBUG { println!("{:?}", &buf); }
        return self.socket.send(&buf);
    }
}

pub fn unpack_player_data(buf: &mut [u8; PACKET_SIZE])
                          -> Result<(f32, f32, bool, i32, i32, u32, u32), String> {
    let mut xpos: [u8; 4] = [0; 4];
    for i in 0..4 {
        xpos[i] = buf[i];
    }

    let mut ypos: [u8; 4] = [0; 4];
    for i in 4..8 {
        ypos[i-4] = buf[i];
    }
    let mut flip: [u8; 4] = [0; 4];
    for i in 8..12 {
        flip[i-8] = buf[i];
    }

    let mut ax :[u8; 4] = [0; 4];
    for i in 24..28 {
        ax[i-24] = buf[i];
    }

    let mut ay :[u8; 4] = [0; 4];
    for i in 28..32 {
        ay[i-28] = buf[i];
    }

    let mut aw :[u8; 4] = [0; 4];
    for i in 32..36 {
        aw[i-32] = buf[i];
    }

    let mut ah :[u8; 4] = [0; 4];
    for i in 36..40 {
        ah[i-36] = buf[i];
    }

    let x = f32::from_le_bytes(xpos);
    let y = f32::from_le_bytes(ypos);
    let flip = u32::from_le_bytes(flip);
    if flip != 1 && flip != 0 {
        return Err(String::from("Error: player flip is neither 1 nor 0"));
    }
    let flip = flip == 1;
    let ax = i32::from_le_bytes(ax);
    let ay = i32::from_le_bytes(ay);
    let aw = u32::from_le_bytes(aw);
    let ah = u32::from_le_bytes(ah);
    // debug
    let tup = (x, y, flip, ax, ay, aw, ah);
    if DEBUG {
        println!("tup = {:?}", tup);
        println!("buf = {:?}", buf);
    }
    Ok(tup)
}

// refactor to make safe -- return result
pub fn unpack_portal_data(buf: &mut [u8; PACKET_SIZE]) -> (f32, f32, f32) {
    let mut xpos: [u8; 4] = [0; 4];
    for i in 12..16 {
        xpos[i-12] = buf[i];
    }

    let mut ypos: [u8; 4] = [0; 4];
    for i in 16..20 {
        ypos[i-16] = buf[i];
    }

    let mut rotation: [u8; 4] = [0; 4];
    for i in 20..24 {
        rotation[i-20] = buf[i];
    }

    let x1 = f32::from_le_bytes(xpos);
    let y1 = f32::from_le_bytes(ypos);
    let rotation1 = f32::from_le_bytes(rotation);
   
    (x1,y1,rotation1)
}

pub(crate) fn unpack_block_data(buf: &mut [u8; PACKET_SIZE]) -> (i32, i32, bool){
    let mut block_x: [u8; 4] = [0; 4];
    for i in 40..44 {
        block_x[i-40] = buf[i];
    }

    let mut block_y: [u8; 4] = [0; 4];
    for i in 44..48 {
        block_y[i-44] = buf[i];
    }

    let mut carried: [u8; 4] = [0; 4];
    for i in 48..52 {
        carried[i-48] = buf[i];
    }

    let block_x = i32::from_le_bytes(block_x);
    let block_y = i32::from_le_bytes(block_y);
    let carried = i32::from_le_bytes(carried);
    let carried = match carried {
        0 => false,
        _ => true,
    };

    (block_x, block_y, carried)
}
