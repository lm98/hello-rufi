use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;
use std::time::Duration;
use rf_core::context::Context;
use rf_core::export::Export;
use rf_core::lang::execution::round;
use rf_core::vm::round_vm::RoundVM;
use crate::mailbox::{AsStates, Mailbox};
use crate::message::Message;
use crate::network::{Network, NetworkUpdate};

/// This struct represents the platform on which the program is executed
pub struct Platform {
    mailbox: Box<dyn Mailbox>,
    network: Box<dyn Network>,
    context: Context,
    nbrs: Vec<i32>,
}

impl Platform {
    pub fn new(mailbox: Box<dyn Mailbox>, network: Box<dyn Network>, context: Context, nbrs: Option<Vec<i32>>) -> Self {
        if let Some(nbrs) = nbrs {
            Self {
                mailbox,
                network,
                context,
                nbrs,
            }
        } else {
            Self {
                mailbox,
                network,
                context,
                nbrs: vec![],
            }
        }
    }

    /// Runs indefinitely the program on the platform
    ///
    /// # Arguments
    ///
    /// * `program` - The aggregate program to be executed
    ///
    /// # Generic Arguments
    ///
    /// * `P` - The type of the aggregate program, it must be a function that takes a [RoundVM] and returns a [RoundVM] and a result of type `A`
    /// * `A` - The type of the result of the aggregate program
    pub async fn run_forever<P, A>(mut self, program: P) -> Result<(), Box<dyn Error>>
        where
            P: Fn(RoundVM) -> (RoundVM, A) + Copy,
            A: Clone + 'static + FromStr + Display,
    {
        loop {
            single_cycle(&mut self.mailbox, &mut self.network, self.context.clone(), program).await?;
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    fn add_nbr(&mut self, nbr: i32) {
        self.nbrs.push(nbr);
    }

    fn remove_nbr(&mut self, nbr: i32) {
        self.nbrs.retain(|n| n != &nbr);
    }
}

/// Performs a single step of the execution cycle of an aggregate program
///
/// # Arguments
///
/// * `mailbox` - The mailbox of the device
/// * `network` - The network through which the device communicates
/// * `context` - The context of the device
/// * `program` - The aggregate program to be executed
///
/// # Generic Arguments
///
/// * `P` - The type of the aggregate program, it must be a function that takes a [RoundVM] and returns a [RoundVM] and a result of type `A`
/// * `A` - The type of the result of the aggregate program
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - The result of the execution
async fn single_cycle<P, A>(mailbox: &mut Box<dyn Mailbox>, network: &mut Box<dyn Network>, context: Context, program: P) -> Result<(), Box<dyn Error>>
    where
        P: Fn(RoundVM) -> (RoundVM, A),
        A: Clone + 'static + FromStr + Display,
{
    //STEP 1: Setup the aggregate program execution

    // Retrieve the neighbouring exports from the mailbox
    let states = mailbox.messages().as_states();

    //STEP 2: Execute a round
    let context = Context::new(
        context.self_id().clone(),
        context.local_sensors().clone(),
        context.nbr_sensors().clone(),
        states,
    );
    println!("CONTEXT: {:?}", context);
    let mut vm = RoundVM::new(context);
    vm.new_export_stack();
    let (mut vm_, result) = round(vm, program);
    let self_export: Export = vm_.export_data().clone();
    println!("OUTPUT: {}\nEXPORT: {}\n", result, self_export);

    //STEP 3: Publish the export
    let msg = Message::new(vm_.self_id().clone(), self_export, std::time::SystemTime::now());
    let msg_ser = serde_json::to_string(&msg).unwrap();
    network.send(vm_.self_id().clone(), msg_ser).await?;

    //STEP 4: Receive the neighbouring exports from the network
    if let Ok(update) = network.recv().await {
        match update {
            NetworkUpdate::Update { msg } => {
                let msg: Message = serde_json::from_str(&msg).unwrap();
                mailbox.enqueue(msg);
            }
            _ => {}
        }
    }
    Ok(())
}