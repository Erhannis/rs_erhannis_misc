// de/encode messages to length-prefixed checksummed byte streams.

use heapless::{CapacityError, Vec};
use log::{error, trace};
use sha2::{Digest, Sha256};

use crate::utils::to_hex_string;

const MAGIC_BYTE: u8 = 0b10101001; // 0xA9 // Sorta arbitrary, seems harder to get on accident

pub enum TransmissionStatus {
  Complete,
  Partial(usize),
}

pub trait DecoderT {
  fn read<const CAPACITY: usize>(&mut self, buffer: &mut Vec<u8, CAPACITY>) -> Result<(), nb::Error<()>>;
}

pub trait EncoderT {
  fn write(&mut self, msg: &[u8]) -> Result<(), ()>;
}

pub struct Decoder<const LEN_PREFIX_BYTES: usize, const CHECKSUM_BYTES: usize, STATE, const BUF_SIZE: usize> {
  state: STATE,
  incoming_message: Vec<u8, BUF_SIZE>,
  before_rx: Option<fn(&mut STATE)>,
  rx: fn(&mut STATE, &mut [u8]) -> Result<TransmissionStatus, nb::Error<()>>,
  after_rx: Option<fn(&mut STATE)>,
}

pub struct Encoder<const LEN_PREFIX_BYTES: usize, const CHECKSUM_BYTES: usize, STATE> {
  state: STATE,
  before_tx: Option<fn(&mut STATE)>,
  tx: fn(&mut STATE, &[&[u8]]) -> Result<TransmissionStatus, nb::Error<()>>,
  after_tx: Option<fn(&mut STATE)>,
}

//RAINY Document format, maybe version it
/// Important note: this class API handles complete messages, not just streams of bytes.
/// LEN_PREFIX_BYTES means the width of the uint that can encode the length of the message, basically.
/// CHECKSUM_BYTES means the number of bytes in the suffix checksum (assumed at most 32)
/// Total packet length is 2*LEN_PREFIX_BYTES + message_length + CHECKSUM_BYTES

impl <const LEN_PREFIX_BYTES: usize, const CHECKSUM_BYTES: usize, STATE> Encoder<LEN_PREFIX_BYTES, CHECKSUM_BYTES, STATE> {
  pub fn new(state: STATE, before_tx: Option<fn(&mut STATE)>, tx: fn(&mut STATE, &[&[u8]]) -> Result<TransmissionStatus, nb::Error<()>>, after_tx: Option<fn(&mut STATE)>) -> Encoder<LEN_PREFIX_BYTES, CHECKSUM_BYTES, STATE> {
    return Encoder {
      state: state,
      before_tx: before_tx,
      tx: tx,
      after_tx: after_tx,
    };
  }

  /**
   * Wrap `msg` in length prefix and checksums (and whatever other processing is added in the future)
   * and write it to `out`, skipping all the callbacks and such that the full Encoder has.
   * Note that if not `out.len() == calcMsgSize(LEN_PREFIX_BYTES, CHECKSUM_BYTES, msg.len())`,
   * we return Err(()).
   */ //CHECK Maybe Vec out, not [u8]?
  pub fn write_plain(msg: &[u8], out: &mut [u8]) -> Result<(), ()> { //THINK Should probably have proper ok/err types
    // Makes a new Encoder internally to write bytes into `out`
    //RAINY Mmmmaaaybe generic consts, so I can precalc the output size?  Actually, msg.len() maybe not const huh
    let size_out = calcMsgSize(LEN_PREFIX_BYTES, CHECKSUM_BYTES, msg.len());
    if size_out != out.len() {
      return Err(());
    }
    let mut enc: Encoder<LEN_PREFIX_BYTES, CHECKSUM_BYTES, (usize, &mut [u8])> = Encoder {
      state: (0, out),
      before_tx: None,
      tx: |state, buffers| {
        //THINK We might just be able to send each individually, in this case
        let mut offset = state.0;
        for b in buffers {
            let len = b.len();
            //THINK I *believe* this should be guaranteed not to happen, if the other code is correct?
            // if offset + len > MSG_SIZE {
            //   error!("DBG ERR MSG TOO BIG");
            // }
            state.1[offset..offset + len].copy_from_slice(b);
            offset += len;
        }
        state.0 = offset;
        return Ok(TransmissionStatus::Complete);
      },
      after_tx: None,
    };
    return enc.write(msg);
  }
}

//THINK 
pub const fn calcMsgSize(LEN_PREFIX_BYTES: usize, CHECKSUM_BYTES: usize, MSG_BYTES: usize) -> usize {
  //PERIODIC Keep in sync with encoder/decoder
  let mut r: usize = 0;
  r += 1;                // magic byte
  r += LEN_PREFIX_BYTES; // length
  r += LEN_PREFIX_BYTES; // length checksum
  r += MSG_BYTES;        // msg
  r += CHECKSUM_BYTES;   // checksum
  return r;
}

impl <const LEN_PREFIX_BYTES: usize, const CHECKSUM_BYTES: usize, STATE> EncoderT for Encoder<LEN_PREFIX_BYTES, CHECKSUM_BYTES, STATE> {
  /*
  //THINK It might be nice if we could figure out a way to pass back data without first having to know how much we need
  //THINK Should we also incorporate the "offset" thing somehow?
  //THINK Should the read functions append, or overwrite, or what?  (atm overwriting)
  */

  //RAINY It's kinda weird passing baud and delay in; maybe bundle them up or pass them in another way.  BAUD's a const, in main....
  fn write(&mut self, msg: &[u8]) -> Result<(), ()> { //THINK Should probably have proper ok/err types
    trace!("-->den.write");
    //DUMMY Error correction, retransmission
    //CHECK Little or big endian?  Optionize?
    //RAINY Make non-blocking?
    if msg.len() >= (1 << (8*LEN_PREFIX_BYTES)) {
      trace!("<--den.write");
      return Err(());
    }

    let mut len_buf: [u8; LEN_PREFIX_BYTES] = [0; LEN_PREFIX_BYTES];
    let mut l = msg.len(); // Does not include the checksum
    for i in (0..LEN_PREFIX_BYTES).rev() { //CHECK Is this notably slow?  I'd hope not.
      len_buf[i] = l as u8;
      l = l >> 8;
    }

    let len_hash = Sha256::digest(len_buf);
    let len_checksum = &len_hash[0..LEN_PREFIX_BYTES];

    let msg_hash = Sha256::digest(msg); //CHECK Should I hash the msg, or the entire preceding packet?
    let msg_checksum = &msg_hash[0..CHECKSUM_BYTES];

    match &mut self.before_tx {
      Some(f) => f(&mut self.state),
      None => (),
    };
    match (self.tx)(&mut self.state, &[
      &[MAGIC_BYTE],
      &len_buf,
      &len_checksum,
      msg,
      &msg_checksum,
    ]) {
        Ok(TransmissionStatus::Complete) => (),
        Ok(TransmissionStatus::Partial(_n)) => error!("partial tx not yet handled"),
        Err(_) => error!("tx error, not handled"),
    };
    match &mut self.after_tx {
      Some(f) => f(&mut self.state),
      None => (),
    };
    let h = to_hex_string(&[MAGIC_BYTE]);
    trace!("den.write wrote {}", h.as_str());
    let h = to_hex_string(&len_buf);
    trace!("den.write wrote {}", h.as_str());
    let h = to_hex_string(&len_checksum);
    trace!("den.write wrote {}", h.as_str());
    let h = to_hex_string(msg);
    trace!("den.write wrote {}", h.as_str());
    let h = to_hex_string(&msg_checksum);
    trace!("den.write wrote {}", h.as_str());
    trace!("<--den.write");
    return Ok(());
  }
}

/// See Encoder
impl <const LEN_PREFIX_BYTES: usize, const CHECKSUM_BYTES: usize, STATE, const BUF_SIZE: usize> Decoder<LEN_PREFIX_BYTES, CHECKSUM_BYTES, STATE, BUF_SIZE> {
  pub fn new(state: STATE, before_rx: Option<fn(&mut STATE)>, rx: fn(&mut STATE, &mut [u8]) -> Result<TransmissionStatus, nb::Error<()>>, after_rx: Option<fn(&mut STATE)>) -> Decoder<LEN_PREFIX_BYTES, CHECKSUM_BYTES, STATE, BUF_SIZE> {
    return Decoder {
      state: state,
      incoming_message: Vec::new(),
      before_rx: before_rx,
      rx: rx,
      after_rx: after_rx,
    };
  }

  /// STATE is input not yet processed
  /**
   * Returns a Decoder that you can .add() data to.
   * //RAINY Currently you have to manually specify the state for Decoder, sorry.  Depending on the side you're specifying it on, either Vec<u8, BUF_SIZE>, or about anything at all, will do.
   */
  pub fn new_plain() -> Decoder<LEN_PREFIX_BYTES, CHECKSUM_BYTES, Vec<u8, BUF_SIZE>, BUF_SIZE> {
    let d = Decoder {
      state: Vec::<u8, BUF_SIZE>::new(),
      incoming_message: Vec::new(),
      before_rx: None,
      // Note: Decoder calls rx repeatedly with a buffer of the size it wants.
      rx: |state, buffer| -> Result<TransmissionStatus, nb::Error<()>> { // Rx
          if state.len() < buffer.len() {
            // We have less data than we want
            let n = state.len();
            if n == 0 {
              return Err(nb::Error::WouldBlock);
            }
            buffer[0..n].copy_from_slice(state);
            state.clear();
            return Ok(TransmissionStatus::Partial(n));
          } else if state.len() > buffer.len() {
            // We have more data than we want
            let n = buffer.len();
            buffer.copy_from_slice(state);
            state.drain(0..n); //LEAK I wonder if there's a more efficient way?
            return Ok(TransmissionStatus::Complete);
          } else {
            // We have exactly as much data as we want
            let n = state.len();
            buffer[0..n].copy_from_slice(state);
            state.clear();
            return Ok(TransmissionStatus::Complete);
          }
        },
      after_rx: None,
    };
    return d;
  }
}

impl <const LEN_PREFIX_BYTES: usize, const CHECKSUM_BYTES: usize, const BUF_SIZE: usize> Decoder<LEN_PREFIX_BYTES, CHECKSUM_BYTES, Vec<u8,BUF_SIZE>, BUF_SIZE> {
  /**
   * Copies `input` onto the pending buffer in `state`.  Returns error if out of space.
   */
  pub fn add(&mut self, input: &[u8]) -> Result<(), CapacityError> {
    return self.state.extend_from_slice(input);
  }

  // /**
  //  * Calls `add` and then `read`.
  //  */
  // pub fn read_plain<'a, const CAPACITY: usize>(&mut self, input: &[u8], msg: &'a mut Vec<u8, CAPACITY>) -> Result<Option<&'a Vec<u8, CAPACITY>>, nb::Error<()>> {
  //   // Having trouble with having two error types
  //   match self.add(input) {
  //     Ok(()) => (),
  //     Err(e) => return Err(nb::Error::Other(e)),
  //   }
  //   match self.read(msg) {
  //     Ok(()) => {
  //       return Ok(Some(msg));
  //     },
  //     Err(e) => return Err(e),
  //   }
  //   // return Ok(Some(msg));
  // }

  /**
   * Removes current input from buffer (clearing the buffer) and returns it.
   */
  pub fn recover_input(&mut self) -> Vec<u8,BUF_SIZE> {
    return core::mem::replace(&mut self.state, Vec::new());
  }

  //RAINY Should this be on the trait?
  //RAINY Should we provide a way to reset STATE?
  /**
   * Clears `incoming_message`, resetting the built-in internal state of Decoder.
   * Notably, does not reset the STATE field, which the built-in code knows nothing about.
   */
  pub fn clear(&mut self) {
    self.incoming_message.clear();
  }
}

impl <const LEN_PREFIX_BYTES: usize, const CHECKSUM_BYTES: usize, STATE, const BUF_SIZE: usize> DecoderT for Decoder<LEN_PREFIX_BYTES, CHECKSUM_BYTES, STATE, BUF_SIZE> {
  // Returns error if error, else overwrites buffer with received message and sets buffer.length accordingly
  //THINK If a message fails validation, should I return an error?
  //        I think the eventual goal is that we shall handle all such problems
  fn read<const CAPACITY: usize>(&mut self, buffer: &mut Vec<u8, CAPACITY>) -> Result<(), nb::Error<()>> { //THINK Should it return Err(size of waiting message) if too big, or st?
    trace!("-->den.read");
    match &mut self.before_rx {
      Some(f) => f(&mut self.state),
      None => (),
    };

    /*
    So, this goes through the steps and compares against how much data we already have, to figure out where in the process we are,
    and resume from there, returning Err(WouldBlock) when would block.
    //LEAK I imagine this wastes time?  Can we keep state better?
     */

    // Looping so I can restart on validation failure
    'readloop: loop {
      let mut benchmark = 0; // For tracking target index

      // Find magic byte
      benchmark += 1;
      if self.incoming_message.len() < benchmark {
        //CHECK I don't really like using magic bytes, I think I'd prefer to just check all alignments and rely on checksums
        let mut one_byte = [0u8; 1];
        while MAGIC_BYTE != (match (self.rx)(&mut self.state, &mut one_byte) {
          Ok(TransmissionStatus::Complete) => {
            one_byte[0]
          },
          Ok(TransmissionStatus::Partial(n)) => {
            error!("Weird; got Partial({}) for [u8; 1]?", n);
            return Err(nb::Error::WouldBlock);
          },
          Err(e) => return Err(e),
        }) {};
        if self.incoming_message.push(MAGIC_BYTE).is_err() {
          return Err(nb::Error::Other(())); //RAINY Specify an error
        }
        // Found magic byte
      }

      let mut load_bytes = |incoming_message: &mut Vec<u8, BUF_SIZE>, mut buf: &mut [u8]| -> Result<_, nb::Error<()>> { //THINK Might be clearer to do `count` instead of buf; not really necessary I think.  Can we do const params?
        match (self.rx)(&mut self.state, &mut buf) {
          Ok(TransmissionStatus::Complete) => {
            for &mut b in buf {
              if incoming_message.push(b).is_err() {
                return Err(nb::Error::Other(())); //RAINY Specify an error
              }
            }
          },
          Ok(TransmissionStatus::Partial(n)) => {
            for &b in buf[..n].iter() {
              if incoming_message.push(b).is_err() {
                return Err(nb::Error::Other(())); //RAINY Specify an error
              }
            }
            return Err(nb::Error::WouldBlock);
          },
          Err(e) => return Err(e),
        };
        return Ok(TransmissionStatus::Complete);
      };

      // Read length
      benchmark += LEN_PREFIX_BYTES;
      if self.incoming_message.len() < benchmark {
        let missing: usize = benchmark - self.incoming_message.len();
        let mut buf: [u8;LEN_PREFIX_BYTES] = [0;LEN_PREFIX_BYTES]; // This buffer probably isn't necessary, buuuut....
        load_bytes(&mut self.incoming_message, &mut buf[..missing])?;
      }
      let mut len_buf = [0_u8;LEN_PREFIX_BYTES]; //LEAK It would be nicer if we could skip the initialization step
      len_buf.copy_from_slice(&self.incoming_message[(benchmark-LEN_PREFIX_BYTES)..benchmark]);

      // Read length checksum
      benchmark += LEN_PREFIX_BYTES;
      if self.incoming_message.len() < benchmark {
        let missing: usize = benchmark - self.incoming_message.len();
        //THINK Note the length checksum is LEN_PREFIX_BYTES long, not CHECKSUM_BYTES long; kinda confusing
        let mut buf: [u8;LEN_PREFIX_BYTES] = [0;LEN_PREFIX_BYTES]; // Ditto
        load_bytes(&mut self.incoming_message, &mut buf[..missing])?;
        let len_checksum = &self.incoming_message[(benchmark-LEN_PREFIX_BYTES)..benchmark];

        // Check length checksum
        let len_hash = Sha256::digest(len_buf);
        let len_checksum_calc = &len_hash[0..LEN_PREFIX_BYTES]; //DITTO Confusing name
        for i in 0..LEN_PREFIX_BYTES {
          if len_checksum[i] != len_checksum_calc[i] {
            let a = to_hex_string(&len_checksum);
            let b = to_hex_string(len_checksum_calc);
            error!("den.read: Incoming message failed length checksum {} != {}", a.as_str(), b.as_str());
            self.incoming_message.clear();
            continue 'readloop;
          }
        }
      }
      
      // Passed length checksum; verify we have enough space
      let mut len: usize = 0;
      for b in len_buf {
        len = (len << 8) ^ (b as usize);
      }
      match buffer.resize_default(len) {
        Ok(_) => {
          // Pass through
        },
        Err(_) => {
          error!("den.read: Incoming message too big(?) {} > {}, dropped", len, CAPACITY); //DUMMY I think this encountered an example of like, the magic byte was in the message, and it got off track, and failed to recover, always parsing the msg wrong
          self.incoming_message.clear();
          continue 'readloop;
        },
      };
      
      // Passed length verification; read message
      benchmark += len;
      if self.incoming_message.len() < benchmark {
        let missing: usize = benchmark - self.incoming_message.len();
        let mut buf: [u8;CAPACITY] = [0;CAPACITY]; // Ditto
        load_bytes(&mut self.incoming_message, &mut buf[..missing])?;
      }
      let mut msg_buf0 = [0_u8;CAPACITY]; //LEAK //DITTO Esp here
      let msg_buf = &mut msg_buf0[0..len];
      msg_buf.copy_from_slice(&self.incoming_message[(benchmark-len)..benchmark]);

      // Read msg checksum
      benchmark += CHECKSUM_BYTES;
      if self.incoming_message.len() < benchmark {
        let missing: usize = benchmark - self.incoming_message.len();
        let mut buf: [u8;CHECKSUM_BYTES] = [0;CHECKSUM_BYTES]; // Ditto
        load_bytes(&mut self.incoming_message, &mut buf[..missing])?;
      }
      let mut msg_checksum = [0_u8;CHECKSUM_BYTES]; //LEAK It would be nicer if we could skip the initialization step
      msg_checksum.copy_from_slice(&self.incoming_message[(benchmark-CHECKSUM_BYTES)..benchmark]);

      // Check message checksum
      let msg_hash = Sha256::digest(&msg_buf);
      let msg_checksum_calc = &msg_hash[0..CHECKSUM_BYTES];
      for i in 0..CHECKSUM_BYTES {
        if msg_checksum[i] != msg_checksum_calc[i] {
          let a = to_hex_string(&msg_checksum);
          let b = to_hex_string(msg_checksum_calc);
          error!("den.read: Incoming message failed msg checksum {} != {}", a.as_str(), b.as_str());
          //THINK Error correction?
          self.incoming_message.clear();
          continue 'readloop;
        }
      }

      match &mut self.after_rx {
        Some(f) => f(&mut self.state),
        None => (),
      };

      buffer.copy_from_slice(msg_buf); // `buffer` was already resized to len, as is msg_buf
      self.incoming_message.clear();
      trace!("<--den.read");
      return Ok(());
    }
  }
}
