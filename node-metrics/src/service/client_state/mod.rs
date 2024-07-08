use super::{
    client_id::ClientId,
    client_message::InternalClientMessage,
    data_state::{DataState, NodeIdentity},
    server_message::ServerMessage,
};
use async_std::sync::{RwLock, RwLockWriteGuard};
use bitvec::vec::BitVec;
use futures::{
    channel::mpsc::{SendError, Sender},
    SinkExt, Stream, StreamExt,
};
use hotshot_query_service::explorer::{BlockDetail, ExplorerHistograms};
use sequencer::SeqTypes;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

/// ClientState represents the service state of the connected clients.
/// It maintains and represents the connected clients, and their subscriptions.
// This state is meant to be managed in a separate thread that assists with
// processing and updating of individual client states.
pub struct ClientState {
    client_id: ClientId,
    sender: Sender<ServerMessage>,
}

impl ClientState {
    /// Create a new ClientState with the given client_id and receiver.
    pub fn new(client_id: ClientId, sender: Sender<ServerMessage>) -> Self {
        Self { client_id, sender }
    }

    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub fn sender(&self) -> &Sender<ServerMessage> {
        &self.sender
    }
}

/// [ClientThreadState] represents the state of all of the active client
/// connections connected to the service. This state governs which clients
/// are connected, and what subscriptions they have setup.
pub struct ClientThreadState {
    clients: HashMap<ClientId, ClientState>,
    subscribed_latest_block: HashSet<ClientId>,
    subscribed_node_identity: HashSet<ClientId>,
    subscribed_voters: HashSet<ClientId>,
    connection_id_counter: ClientId,
}

impl ClientThreadState {
    pub fn new(
        clients: HashMap<ClientId, ClientState>,
        subscribed_latest_block: HashSet<ClientId>,
        subscribed_node_identity: HashSet<ClientId>,
        subscribed_voters: HashSet<ClientId>,
        connection_id_counter: ClientId,
    ) -> Self {
        Self {
            clients,
            subscribed_latest_block,
            subscribed_node_identity,
            subscribed_voters,
            connection_id_counter,
        }
    }
}

/// [drop_client_client_thread_state_write_guard] is a utility function for
/// cleaning up the [ClientThreadState]
fn drop_client_client_thread_state_write_guard(
    client_id: &ClientId,
    client_thread_state_write_guard: &mut RwLockWriteGuard<ClientThreadState>,
) -> Option<ClientState> {
    let client = client_thread_state_write_guard.clients.remove(client_id);
    client_thread_state_write_guard
        .subscribed_latest_block
        .remove(client_id);
    client_thread_state_write_guard
        .subscribed_node_identity
        .remove(client_id);

    client
}

/// [drop_client_no_lock_guard] is a utility function for cleaning up the [ClientThreadState]
/// when a client is detected as disconnected.
async fn drop_client_no_lock_guard(
    client_id: &ClientId,
    client_thread_state: Arc<RwLock<ClientThreadState>>,
) -> Option<ClientState> {
    let mut client_thread_state_write_lock_guard = client_thread_state.write().await;

    drop_client_client_thread_state_write_guard(
        client_id,
        &mut client_thread_state_write_lock_guard,
    )
}

/// [HandleConnectedError] represents the scope of errors that can be
/// returned from the [handle_client_message_connected] function.
#[derive(Debug)]
pub enum HandleConnectedError {
    ClientSendError(SendError),
}

impl std::fmt::Display for HandleConnectedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandleConnectedError::ClientSendError(err) => {
                write!(f, "handle connected error: client send error: {}", err)
            }
        }
    }
}

impl std::error::Error for HandleConnectedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HandleConnectedError::ClientSendError(err) => Some(err),
        }
    }
}

/// [handle_client_message_connected] is a function that processes the client
/// message to connect a client to the service.
pub async fn handle_client_message_connected(
    mut sender: Sender<ServerMessage>,
    client_thread_state: Arc<RwLock<ClientThreadState>>,
) -> Result<ClientId, HandleConnectedError> {
    let mut client_thread_state_write_lock_guard = client_thread_state.write().await;

    client_thread_state_write_lock_guard.connection_id_counter += 1;
    let client_id = client_thread_state_write_lock_guard.connection_id_counter;

    client_thread_state_write_lock_guard.clients.insert(
        client_id,
        ClientState {
            client_id,
            sender: sender.clone(),
        },
    );

    // Explicitly unlock
    drop(client_thread_state_write_lock_guard);

    // Send the client their new id.
    if let Err(err) = sender.send(ServerMessage::YouAre(client_id)).await {
        // We need to remove drop the client now.
        drop_client_no_lock_guard(&client_id, client_thread_state.clone()).await;
        return Err(HandleConnectedError::ClientSendError(err));
    }

    Ok(client_id)
}

/// [handle_client_message_disconnected] is a function that processes the client
/// message to disconnect a client from the service.
pub async fn handle_client_message_disconnected(
    client_id: ClientId,
    client_thread_state: Arc<RwLock<ClientThreadState>>,
) {
    // We might receive an implicit disconnect when attempting to
    // send a message, as the receiving channel might be closed.
    drop_client_no_lock_guard(&client_id, client_thread_state.clone()).await;
}

/// [handle_client_message_subscribe_latest_block] is a function that processes
/// the client message to subscribe to the latest block stream.
pub async fn handle_client_message_subscribe_latest_block(
    client_id: ClientId,
    client_thread_state: Arc<RwLock<ClientThreadState>>,
) {
    let mut client_thread_state_write_lock_guard = client_thread_state.write().await;

    client_thread_state_write_lock_guard
        .subscribed_latest_block
        .insert(client_id);

    // Explicitly unlock
    drop(client_thread_state_write_lock_guard);
}

/// [handle_client_message_subscribe_node_identity] is a function that processes
/// the client message to subscribe to the node identity stream.
pub async fn handle_client_message_subscribe_node_identity(
    client_id: ClientId,
    client_thread_state: Arc<RwLock<ClientThreadState>>,
) {
    let mut client_thread_state_write_lock_guard = client_thread_state.write().await;

    client_thread_state_write_lock_guard
        .subscribed_node_identity
        .insert(client_id);

    // Explicitly unlock
    drop(client_thread_state_write_lock_guard);
}

/// [handle_client_message_subscribe_voters] is a function that processes
/// the client message to subscribe to the voters bitvecs.
pub async fn handle_client_message_subscribe_voters(
    client_id: ClientId,
    client_thread_state: Arc<RwLock<ClientThreadState>>,
) {
    let mut client_thread_state_write_lock_guard = client_thread_state.write().await;

    client_thread_state_write_lock_guard
        .subscribed_voters
        .insert(client_id);

    // Explicitly unlock
    drop(client_thread_state_write_lock_guard);
}

/// [HandleRequestBlocksSnapshotsError] represents the scope of errors that can
/// be returned from the [handle_client_message_request_blocks_snapshot] function.
#[derive(Debug)]
pub enum HandleRequestBlocksSnapshotsError {
    ClientSendError(SendError),
}

impl std::fmt::Display for HandleRequestBlocksSnapshotsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandleRequestBlocksSnapshotsError::ClientSendError(err) => {
                write!(
                    f,
                    "handle request blocks snapshot error: client send error:: {}",
                    err
                )
            }
        }
    }
}

impl std::error::Error for HandleRequestBlocksSnapshotsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HandleRequestBlocksSnapshotsError::ClientSendError(err) => Some(err),
        }
    }
}

/// [handle_client_message_request_blocks_snapshot] is a function that processes
/// the client message request for a blocks snapshot.
pub async fn handle_client_message_request_blocks_snapshot(
    client_id: ClientId,
    data_state: Arc<RwLock<DataState>>,
    client_thread_state: Arc<RwLock<ClientThreadState>>,
) -> Result<(), HandleRequestBlocksSnapshotsError> {
    let (client_thread_state_read_lock_guard, data_state_read_lock_guard) =
        futures::join!(client_thread_state.read(), data_state.read());

    let latest_blocks = data_state_read_lock_guard
        .latest_blocks()
        .map(|block| BlockDetail {
            hash: block.hash,
            proposer_id: block.proposer_id,
            height: block.height,
            size: block.size,
            time: block.time,
            num_transactions: block.num_transactions,
            fee_recipient: block.fee_recipient,
            block_reward: block.block_reward.clone(),
        })
        .collect::<Vec<BlockDetail<SeqTypes>>>();

    if let Some(client) = client_thread_state_read_lock_guard.clients.get(&client_id) {
        let mut sender = client.sender.clone();
        if let Err(err) = sender
            .send(ServerMessage::BlocksSnapshot(Arc::new(latest_blocks)))
            .await
        {
            drop_client_no_lock_guard(&client_id, client_thread_state.clone()).await;
            return Err(HandleRequestBlocksSnapshotsError::ClientSendError(err));
        }
    }

    Ok(())
}

/// [HandleRequestNodeIdentitySnapshotError] represents the scope of errors that
/// can be returned from the [handle_client_message_request_node_identity_snapshot]
/// function.
#[derive(Debug)]
pub enum HandleRequestNodeIdentitySnapshotError {
    ClientSendError(SendError),
}

impl std::fmt::Display for HandleRequestNodeIdentitySnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandleRequestNodeIdentitySnapshotError::ClientSendError(err) => {
                write!(
                    f,
                    "handle request node identity snapshot error: client send error: {}",
                    err
                )
            }
        }
    }
}

impl std::error::Error for HandleRequestNodeIdentitySnapshotError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HandleRequestNodeIdentitySnapshotError::ClientSendError(err) => Some(err),
        }
    }
}

/// [handle_client_message_request_node_identity_snapshot] is a function that
/// processes the client message request for a node identity snapshot.
pub async fn handle_client_message_request_node_identity_snapshot(
    client_id: ClientId,
    data_state: Arc<RwLock<DataState>>,
    client_thread_state: Arc<RwLock<ClientThreadState>>,
) -> Result<(), HandleRequestNodeIdentitySnapshotError> {
    // Let's send the current Blocks Snapshot to the client
    let (client_thread_state_read_lock_guard, data_state_read_lock_guard) =
        futures::join!(client_thread_state.read(), data_state.read());
    let client_result = client_thread_state_read_lock_guard.clients.get(&client_id);
    if let Some(client) = client_result {
        let mut sender = client.sender.clone();

        // Let's copy the current node identity snapshot and send them
        let nodes = data_state_read_lock_guard
            .node_identity()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();

        if let Err(err) = sender
            .send(ServerMessage::NodeIdentitySnapshot(Arc::new(nodes)))
            .await
        {
            drop(client_thread_state_read_lock_guard);
            drop_client_no_lock_guard(&client_id, client_thread_state.clone()).await;
            return Err(HandleRequestNodeIdentitySnapshotError::ClientSendError(err));
        }

        return Ok(());
    }

    Ok(())
}

/// [HandleRequestHistogramSnapshotError] represents the scope of errors that
/// can be returned from the [handle_client_message_request_histogram_snapshot]
/// function.
#[derive(Debug)]
pub enum HandleRequestHistogramSnapshotError {
    ClientSendError(SendError),
}

impl std::fmt::Display for HandleRequestHistogramSnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandleRequestHistogramSnapshotError::ClientSendError(err) => {
                write!(
                    f,
                    "handle request histogram snapshot error: client send error: {}",
                    err
                )
            }
        }
    }
}

impl std::error::Error for HandleRequestHistogramSnapshotError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HandleRequestHistogramSnapshotError::ClientSendError(err) => Some(err),
        }
    }
}

/// [handle_client_message_request_histogram_snapshot] is a function that
/// processes the client message request for a histogram snapshot.
pub async fn handle_client_message_request_histogram_snapshot(
    client_id: ClientId,
    data_state: Arc<RwLock<DataState>>,
    client_thread_state: Arc<RwLock<ClientThreadState>>,
) -> Result<(), HandleRequestHistogramSnapshotError> {
    // Let's send the current histogram data snapshot to the client
    let (client_thread_state_read_lock_guard, data_state_read_lock_guard) =
        futures::join!(client_thread_state.read(), data_state.read());

    let histogram_data = ExplorerHistograms {
        block_size: data_state_read_lock_guard
            .latest_blocks()
            .skip(1)
            .map(|block| block.size)
            .collect(),
        block_time: data_state_read_lock_guard
            .latest_blocks()
            .skip(1)
            .zip(data_state_read_lock_guard.latest_blocks())
            .map(|(block_i, block_i_sub_1)| {
                (block_i.time.0 - block_i_sub_1.time.0).whole_seconds() as u64
            })
            .collect(),
        block_transactions: data_state_read_lock_guard
            .latest_blocks()
            .skip(1)
            .map(|block| block.num_transactions)
            .collect(),
        block_heights: data_state_read_lock_guard
            .latest_blocks()
            .skip(1)
            .map(|block| block.height)
            .collect(),
    };
    let arc_histogram_data = Arc::new(histogram_data);
    drop(data_state_read_lock_guard);

    if let Some(client) = client_thread_state_read_lock_guard.clients.get(&client_id) {
        let mut sender = client.sender.clone();
        drop(client_thread_state_read_lock_guard);

        if let Err(err) = sender
            .send(ServerMessage::HistogramSnapshot(arc_histogram_data))
            .await
        {
            drop_client_no_lock_guard(&client_id, client_thread_state.clone()).await;
            return Err(HandleRequestHistogramSnapshotError::ClientSendError(err));
        }

        return Ok(());
    }

    Ok(())
}

#[derive(Debug)]
pub enum HandleRequestVotersSnapshotError {
    ClientSendError(SendError),
}

impl std::fmt::Display for HandleRequestVotersSnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandleRequestVotersSnapshotError::ClientSendError(err) => {
                write!(
                    f,
                    "handle request voters snapshot error: client send error: {}",
                    err
                )
            }
        }
    }
}

impl std::error::Error for HandleRequestVotersSnapshotError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HandleRequestVotersSnapshotError::ClientSendError(err) => Some(err),
        }
    }
}

/// [handle_client_message_request_voters_snapshot] is a function that processes
/// the client message request for a voters snapshot.
pub async fn handle_client_message_request_voters_snapshot(
    client_id: ClientId,
    data_state: Arc<RwLock<DataState>>,
    client_thread_state: Arc<RwLock<ClientThreadState>>,
) -> Result<(), HandleRequestVotersSnapshotError> {
    let (client_thread_state_read_lock_guard, data_state_read_lock_guard) =
        futures::join!(client_thread_state.read(), data_state.read());

    let voters_data = data_state_read_lock_guard
        .latest_voters()
        .cloned()
        .collect::<Vec<_>>();

    let voters_data = Arc::new(voters_data);

    if let Some(client) = client_thread_state_read_lock_guard.clients.get(&client_id) {
        let mut sender = client.sender.clone();
        drop(client_thread_state_read_lock_guard);

        if let Err(err) = sender
            .send(ServerMessage::VotersSnapshot(voters_data.clone()))
            .await
        {
            drop_client_no_lock_guard(&client_id, client_thread_state.clone()).await;
            return Err(HandleRequestVotersSnapshotError::ClientSendError(err));
        }

        return Ok(());
    }
    Ok(())
}

/// [ProcessClientMessageError] represents the scope of errors that can be
/// returned from the [process_client_message] function.
#[derive(Debug)]
pub enum ProcessClientMessageError {
    Connected(HandleConnectedError),
    BlocksSnapshot(HandleRequestBlocksSnapshotsError),
    NodeIdentitySnapshot(HandleRequestNodeIdentitySnapshotError),
    HistogramSnapshot(HandleRequestHistogramSnapshotError),
    VotersSnapshot(HandleRequestVotersSnapshotError),
}

impl From<HandleConnectedError> for ProcessClientMessageError {
    fn from(err: HandleConnectedError) -> Self {
        ProcessClientMessageError::Connected(err)
    }
}

impl From<HandleRequestBlocksSnapshotsError> for ProcessClientMessageError {
    fn from(err: HandleRequestBlocksSnapshotsError) -> Self {
        ProcessClientMessageError::BlocksSnapshot(err)
    }
}

impl From<HandleRequestNodeIdentitySnapshotError> for ProcessClientMessageError {
    fn from(err: HandleRequestNodeIdentitySnapshotError) -> Self {
        ProcessClientMessageError::NodeIdentitySnapshot(err)
    }
}

impl From<HandleRequestHistogramSnapshotError> for ProcessClientMessageError {
    fn from(err: HandleRequestHistogramSnapshotError) -> Self {
        ProcessClientMessageError::HistogramSnapshot(err)
    }
}

impl From<HandleRequestVotersSnapshotError> for ProcessClientMessageError {
    fn from(err: HandleRequestVotersSnapshotError) -> Self {
        ProcessClientMessageError::VotersSnapshot(err)
    }
}

impl std::fmt::Display for ProcessClientMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessClientMessageError::Connected(err) => {
                write!(f, "process client message error: connected: {}", err)
            }
            ProcessClientMessageError::BlocksSnapshot(err) => {
                write!(f, "process client message error: blocks snapshot: {}", err)
            }
            ProcessClientMessageError::NodeIdentitySnapshot(err) => {
                write!(
                    f,
                    "process client message error: node identity snapshot: {}",
                    err
                )
            }
            ProcessClientMessageError::HistogramSnapshot(err) => {
                write!(
                    f,
                    "process client message error: histogram snapshot: {}",
                    err
                )
            }
            ProcessClientMessageError::VotersSnapshot(err) => {
                write!(f, "process client message error: voters snapshot: {}", err)
            }
        }
    }
}

impl std::error::Error for ProcessClientMessageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProcessClientMessageError::Connected(err) => Some(err),
            ProcessClientMessageError::BlocksSnapshot(err) => Some(err),
            ProcessClientMessageError::NodeIdentitySnapshot(err) => Some(err),
            ProcessClientMessageError::HistogramSnapshot(err) => Some(err),
            ProcessClientMessageError::VotersSnapshot(err) => Some(err),
        }
    }
}

/// [process_client_message] is a function that processes the client message
/// and processes the message accordingly.
///
/// The [DataState] is provided and is used only as a Read lock to distribute
/// the current state of the system to the clients upon request.
///
/// The [ClientThreadState] is provided as it needs to be updated with new
/// subscriptions / new connections depending on the incoming
/// [InternalClientMessage]
pub async fn process_client_message(
    message: InternalClientMessage,
    data_state: Arc<RwLock<DataState>>,
    client_thread_state: Arc<RwLock<ClientThreadState>>,
) -> Result<(), ProcessClientMessageError> {
    match message {
        InternalClientMessage::Connected(sender) => {
            handle_client_message_connected(sender, client_thread_state).await?;
            Ok(())
        }

        InternalClientMessage::Disconnected(client_id) => {
            handle_client_message_disconnected(client_id, client_thread_state).await;
            Ok(())
        }

        InternalClientMessage::SubscribeLatestBlock(client_id) => {
            handle_client_message_subscribe_latest_block(client_id, client_thread_state).await;
            Ok(())
        }

        InternalClientMessage::SubscribeNodeIdentity(client_id) => {
            handle_client_message_subscribe_node_identity(client_id, client_thread_state).await;
            Ok(())
        }

        InternalClientMessage::SubscribeVoters(client_id) => {
            handle_client_message_subscribe_voters(client_id, client_thread_state).await;
            Ok(())
        }

        InternalClientMessage::RequestBlocksSnapshot(client_id) => {
            handle_client_message_request_blocks_snapshot(
                client_id,
                data_state,
                client_thread_state,
            )
            .await?;
            Ok(())
        }

        InternalClientMessage::RequestNodeIdentitySnapshot(client_id) => {
            handle_client_message_request_node_identity_snapshot(
                client_id,
                data_state,
                client_thread_state,
            )
            .await?;
            Ok(())
        }

        InternalClientMessage::RequestHistogramSnapshot(client_id) => {
            handle_client_message_request_histogram_snapshot(
                client_id,
                data_state,
                client_thread_state,
            )
            .await?;
            Ok(())
        }

        InternalClientMessage::RequestVotersSnapshot(client_id) => {
            handle_client_message_request_voters_snapshot(
                client_id,
                data_state,
                client_thread_state,
            )
            .await?;
            Ok(())
        }
    }
}

/// [clone_block_detail] is a utility function that clones a [BlockDetail]
/// instance.
pub fn clone_block_detail(input: &BlockDetail<SeqTypes>) -> BlockDetail<SeqTypes> {
    BlockDetail {
        hash: input.hash,
        proposer_id: input.proposer_id,
        height: input.height,
        size: input.size,
        time: input.time,
        num_transactions: input.num_transactions,
        fee_recipient: input.fee_recipient,
        block_reward: input.block_reward.clone(),
    }
}

/// [drop_failed_client_sends] is a function that will drop all of the failed
/// client sends from the client thread state.
async fn drop_failed_client_sends(
    client_thread_state: Arc<RwLock<ClientThreadState>>,
    failed_client_sends: Vec<ClientId>,
) {
    // Let's acquire our write lock
    let mut client_thread_state_write_lock_guard = client_thread_state.write().await;

    // We want to drop all of the failed clients.
    // There's an optimization to be had here
    for client_id in failed_client_sends {
        drop_client_client_thread_state_write_guard(
            &client_id,
            &mut client_thread_state_write_lock_guard,
        );
    }
}

/// [handle_received_block_detail] is a function that processes received Block
/// details and will attempt to distribute the message to all of the clients
/// that are subscribed to the latest block stream.
async fn handle_received_block_detail(
    client_thread_state: Arc<RwLock<ClientThreadState>>,
    block_detail: BlockDetail<SeqTypes>,
) {
    let client_thread_state_read_lock_guard = client_thread_state.read().await;

    // These are the clients who are subscribed to the latest blocks, that
    // have an active ClientState within the system.
    let latest_block_subscribers = client_thread_state_read_lock_guard
        .subscribed_latest_block
        .iter()
        .map(|client_id| {
            (
                client_id,
                client_thread_state_read_lock_guard.clients.get(client_id),
            )
        })
        .filter(|(_, client)| client.is_some());

    let arc_block_detail = Arc::new(block_detail);
    // We collect the results of sending the latest block to the clients.
    let client_send_result_future = latest_block_subscribers.map(|(client_id, client)| {
        let arc_block_detail = arc_block_detail.clone();
        async move {
            // This is guaranteed to be a some now
            let client = client.unwrap();
            let mut sender = client.sender.clone();
            let send_result = sender
                .send(ServerMessage::LatestBlock(arc_block_detail))
                .await;

            (client_id, send_result)
        }
    });

    let client_send_results = futures::future::join_all(client_send_result_future).await;

    // These are the clients we failed to send the message to.  We copy these
    // here so we can drop our read lock.
    let failed_client_sends = client_send_results
        .into_iter()
        .filter(|(_, send_result)| send_result.is_err())
        .map(|(client_id, _)| *client_id)
        .collect::<Vec<_>>();

    // Explicitly Drop the read lock.
    drop(client_thread_state_read_lock_guard);

    if failed_client_sends.is_empty() {
        return;
    }

    drop_failed_client_sends(client_thread_state, failed_client_sends).await;
}

/// [handle_received_node_identity] is a function that processes received
/// NodeIdentity and will attempt to distribute the message to all of the
/// clients that are subscribed to the node identity stream.
async fn handle_received_node_identity(
    client_thread_state: Arc<RwLock<ClientThreadState>>,
    node_identity: NodeIdentity,
) {
    let client_thread_state_read_lock_guard = client_thread_state.read().await;

    // These are the clients who are subscribed to the node identities, that
    // have an active ClientState within the system.
    let node_identity_subscribers = client_thread_state_read_lock_guard
        .subscribed_node_identity
        .iter()
        .map(|client_id| {
            (
                client_id,
                client_thread_state_read_lock_guard.clients.get(client_id),
            )
        })
        .filter(|(_, client)| client.is_some());

    let arc_node_identity = Arc::new(node_identity);
    // We collect the results of sending the latest block to the clients.
    let client_send_result_future = node_identity_subscribers.map(|(client_id, client)| {
        let arc_node_identity = arc_node_identity.clone();
        async move {
            // This is guaranteed to be a some now
            let client = client.unwrap();
            let mut sender = client.sender.clone();
            let send_result = sender
                .send(ServerMessage::LatestNodeIdentity(arc_node_identity.clone()))
                .await;

            (client_id, send_result)
        }
    });

    let client_send_results = futures::future::join_all(client_send_result_future).await;

    // These are the clients we failed to send the message to.  We copy these
    // here so we can drop our read lock.
    let failed_client_sends = client_send_results
        .into_iter()
        .filter(|(_, send_result)| send_result.is_err())
        .map(|(client_id, _)| *client_id)
        .collect::<Vec<_>>();

    // Explicitly Drop the read lock.
    drop(client_thread_state_read_lock_guard);

    if failed_client_sends.is_empty() {
        return;
    }

    drop_failed_client_sends(client_thread_state, failed_client_sends).await;
}

/// [handle_received_voters] is a function that processes received voters and
/// will attempt to distribute the message to all of the clients that are
/// subscribed to the voters stream.
async fn handle_received_voters(
    client_thread_state: Arc<RwLock<ClientThreadState>>,
    voters: BitVec<u16>,
) {
    let client_thread_state_read_lock_guard = client_thread_state.read().await;

    // These are the clients who are subscribed to the node identities, that
    // have an active ClientState within the system.
    let node_identity_subscribers = client_thread_state_read_lock_guard
        .subscribed_node_identity
        .iter()
        .map(|client_id| {
            (
                client_id,
                client_thread_state_read_lock_guard.clients.get(client_id),
            )
        })
        .filter(|(_, client)| client.is_some());

    // We collect the results of sending the latest block to the clients.
    let client_send_result_future = node_identity_subscribers.map(|(client_id, client)| {
        let voters = voters.clone();
        async move {
            // This is guaranteed to be a some now
            let client = client.unwrap();
            let mut sender = client.sender.clone();
            let send_result = sender.send(ServerMessage::LatestVoters(voters)).await;

            (client_id, send_result)
        }
    });

    let client_send_results = futures::future::join_all(client_send_result_future).await;

    // These are the clients we failed to send the message to.  We copy these
    // here so we can drop our read lock.
    let failed_client_sends = client_send_results
        .into_iter()
        .filter(|(_, send_result)| send_result.is_err())
        .map(|(client_id, _)| *client_id)
        .collect::<Vec<_>>();

    // Explicitly Drop the read lock.
    drop(client_thread_state_read_lock_guard);

    if failed_client_sends.is_empty() {
        return;
    }

    drop_failed_client_sends(client_thread_state, failed_client_sends).await;
}

/// [process_internal_client_message_stream] is a function that processes the
/// client handling stream. This stream is responsible for managing the state
/// of the connected clients, and their subscriptions.
pub async fn process_internal_client_message_stream<S>(
    mut stream: S,
    data_state: Arc<RwLock<DataState>>,
    client_thread_state: Arc<RwLock<ClientThreadState>>,
) where
    S: Stream<Item = InternalClientMessage> + Unpin,
{
    loop {
        let message_result = stream.next().await;
        let message = if let Some(message) = message_result {
            message
        } else {
            tracing::info!("internal client message handler closed.");
            return;
        };

        if let Err(err) =
            process_client_message(message, data_state.clone(), client_thread_state.clone()).await
        {
            tracing::info!(
                "internal client message processing encountered an error: {}",
                err,
            );
            return;
        }
    }
}

/// [process_distribute_block_detail_handling_stream] is a function that
/// processes the the [Stream] of incoming [BlockDetail] and distributes them
/// to all subscribed clients.
pub async fn process_distribute_block_detail_handling_stream<S>(
    client_thread_state: Arc<RwLock<ClientThreadState>>,
    mut stream: S,
) where
    S: Stream<Item = BlockDetail<SeqTypes>> + Unpin,
{
    loop {
        let block_detail_result = stream.next().await;

        let block_detail = if let Some(block_detail) = block_detail_result {
            block_detail
        } else {
            tracing::info!("block detail stream closed.  shutting down client handling stream.",);
            return;
        };

        handle_received_block_detail(client_thread_state.clone(), block_detail).await
    }
}

/// [process_distribute_node_identity_handling_stream] is a function that
/// processes the the [Stream] of incoming [NodeIdentity] and distributes them
/// to all subscribed clients.
pub async fn process_distribute_node_identity_handling_stream<S>(
    client_thread_state: Arc<RwLock<ClientThreadState>>,
    mut stream: S,
) where
    S: Stream<Item = NodeIdentity> + Unpin,
{
    loop {
        let node_identity_result = stream.next().await;

        let node_identity = if let Some(node_identity) = node_identity_result {
            node_identity
        } else {
            tracing::info!("node identity stream closed.  shutting down client handling stream.",);
            return;
        };

        handle_received_node_identity(client_thread_state.clone(), node_identity).await
    }
}

/// [process_distribute_voters_handling_stream] is a function that processes
/// the the [Stream] of incoming [BitVec] and distributes them to all
/// subscribed clients.
pub async fn process_distribute_voters_handling_stream<S>(
    client_thread_state: Arc<RwLock<ClientThreadState>>,
    mut stream: S,
) where
    S: Stream<Item = BitVec<u16>> + Unpin,
{
    loop {
        let voters_result = stream.next().await;

        let voters = if let Some(voters) = voters_result {
            voters
        } else {
            tracing::info!("voters stream closed.  shutting down client handling stream.",);
            return;
        };

        handle_received_voters(client_thread_state.clone(), voters).await
    }
}

#[cfg(test)]
pub mod tests {
    use super::{process_internal_client_message_stream, ClientThreadState};
    use crate::service::{
        client_id::ClientId,
        client_message::InternalClientMessage,
        client_state::{
            process_distribute_block_detail_handling_stream,
            process_distribute_node_identity_handling_stream,
            process_distribute_voters_handling_stream,
        },
        data_state::{
            create_block_detail_from_leaf, process_leaf_stream, DataState, LocationDetails,
            NodeIdentity,
        },
        server_message::ServerMessage,
    };
    use async_std::{prelude::FutureExt, sync::RwLock};
    use bitvec::vec::BitVec;
    use futures::{channel::mpsc, SinkExt, StreamExt};
    use hotshot_types::{signature_key::BLSPubKey, traits::signature_key::SignatureKey};
    use sequencer::{Leaf, NodeState, ValidatedState};
    use std::{
        net::{IpAddr, Ipv4Addr},
        sync::Arc,
        time::Duration,
    };

    pub fn create_test_client_thread_state() -> ClientThreadState {
        ClientThreadState {
            clients: Default::default(),
            subscribed_latest_block: Default::default(),
            subscribed_node_identity: Default::default(),
            subscribed_voters: Default::default(),
            connection_id_counter: ClientId::from_count(1),
        }
    }

    pub fn create_test_data_state() -> (NodeIdentity, NodeIdentity, NodeIdentity, DataState) {
        let node_1 = {
            let (pub_key, _) = BLSPubKey::generated_from_seed_indexed([0; 32], 0);
            NodeIdentity::new(
                pub_key,
                Some("a".to_string()),
                Some(Default::default()),
                Some(vec![IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))]),
                Some("company".to_string()),
                Some(LocationDetails::new((0.0, 0.0), "US".to_string())),
                Some("Windows 11".to_string()),
                Some("espresso".to_string()),
                Some("residential".to_string()),
            )
        };

        let node_2 = {
            let (pub_key, _) = BLSPubKey::generated_from_seed_indexed([0; 32], 1);
            NodeIdentity::new(
                pub_key,
                Some("b".to_string()),
                Some(Default::default()),
                Some(vec![IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2))]),
                Some("company".to_string()),
                Some(LocationDetails::new((0.0, 0.0), "US".to_string())),
                Some("Windows 11".to_string()),
                Some("espresso".to_string()),
                Some("residential".to_string()),
            )
        };

        let node_3 = {
            let (pub_key, _) = BLSPubKey::generated_from_seed_indexed([0; 32], 2);
            NodeIdentity::new(
                pub_key,
                Some("b".to_string()),
                Some(Default::default()),
                Some(vec![IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3))]),
                Some("company".to_string()),
                Some(LocationDetails::new((0.0, 0.0), "US".to_string())),
                Some("Windows 11".to_string()),
                Some("espresso".to_string()),
                Some("residential".to_string()),
            )
        };

        let mut data_state: DataState = Default::default();
        data_state.add_node_identity(node_1.clone());
        data_state.add_node_identity(node_2.clone());
        data_state.add_node_identity(node_3.clone());

        (node_1, node_2, node_3, data_state)
    }

    #[async_std::test]
    async fn test_client_handling_stream_task_shutdown() {
        let (_, _, _, data_state) = create_test_data_state();
        let client_thread_state = Arc::new(RwLock::new(create_test_client_thread_state()));
        let data_state = Arc::new(RwLock::new(data_state));

        let (mut internal_client_message_sender, internal_client_message_receiver) =
            mpsc::channel(1);
        let process_internal_client_message_handle: async_std::task::JoinHandle<()> =
            async_std::task::spawn(process_internal_client_message_stream(
                internal_client_message_receiver,
                data_state,
                client_thread_state,
            ));

        // disconnect the last internal client message sender
        internal_client_message_sender.disconnect();

        // Join the async task.
        if let Err(timeout_error) = process_internal_client_message_handle
            .timeout(Duration::from_millis(200))
            .await
        {
            panic!(
                "process_internal_client_message_handle did not complete in time, error: {}",
                timeout_error
            );
        }
    }

    #[async_std::test]
    async fn test_process_client_handling_stream_request_latest_voters_snapshot() {
        let (_, _, _, mut data_state) = create_test_data_state();
        let client_thread_state = Arc::new(RwLock::new(create_test_client_thread_state()));
        let voters_1 = BitVec::from_vec(vec![0x55]);
        let voters_2 = BitVec::from_vec(vec![0xAA]);
        data_state.add_latest_voters(voters_1.clone());
        data_state.add_latest_voters(voters_2.clone());

        let data_state = Arc::new(RwLock::new(data_state));

        let (internal_client_message_sender, internal_client_message_receiver) = mpsc::channel(1);
        let (server_message_sender_1, mut server_message_receiver_1) = mpsc::channel(1);
        let (server_message_sender_2, mut server_message_receiver_2) = mpsc::channel(1);
        let process_internal_client_message_handle =
            async_std::task::spawn(process_internal_client_message_stream(
                internal_client_message_receiver,
                data_state,
                client_thread_state,
            ));

        // Send a Connected Message to the server
        let mut internal_client_message_sender_1 = internal_client_message_sender.clone();
        assert_eq!(
            internal_client_message_sender_1
                .send(InternalClientMessage::Connected(server_message_sender_1))
                .await,
            Ok(())
        );

        assert_eq!(
            server_message_receiver_1.next().await,
            Some(ServerMessage::YouAre(ClientId::from_count(2))),
        );

        let client_1_id = ClientId::from_count(2);

        let mut internal_client_message_sender_2 = internal_client_message_sender;
        assert_eq!(
            internal_client_message_sender_2
                .send(InternalClientMessage::Connected(server_message_sender_2))
                .await,
            Ok(())
        );

        assert_eq!(
            server_message_receiver_2.next().await,
            Some(ServerMessage::YouAre(ClientId::from_count(3))),
        );

        assert_eq!(
            internal_client_message_sender_1
                .send(InternalClientMessage::RequestVotersSnapshot(client_1_id))
                .await,
            Ok(()),
        );

        assert_eq!(
            server_message_receiver_1.next().await,
            Some(ServerMessage::VotersSnapshot(Arc::new(vec![
                voters_1, voters_2
            ]))),
        );

        // disconnect the internal client message sender
        internal_client_message_sender_1.disconnect();
        internal_client_message_sender_2.disconnect();

        // The server message receiver should be shutdown, and should return
        // nothing further
        assert_eq!(server_message_receiver_1.next().await, None);
        assert_eq!(server_message_receiver_2.next().await, None);

        if let Err(timeout_error) = process_internal_client_message_handle
            .timeout(Duration::from_millis(200))
            .await
        {
            panic!(
                "process_internal_client_message_handle did not complete in time, error: {}",
                timeout_error
            );
        }
    }

    #[async_std::test]
    #[cfg(feature = "testing")]
    async fn test_process_client_handling_stream_request_latest_blocks_snapshot() {
        use super::clone_block_detail;
        use crate::service::data_state::create_block_detail_from_leaf;

        let (_, _, _, mut data_state) = create_test_data_state();
        let client_thread_state = Arc::new(RwLock::new(create_test_client_thread_state()));
        let leaf_1 = Leaf::genesis(&ValidatedState::default(), &NodeState::mock()).await;
        let block_1 = create_block_detail_from_leaf(&leaf_1);
        data_state.add_latest_block(clone_block_detail(&block_1));

        let data_state = Arc::new(RwLock::new(data_state));

        let (internal_client_message_sender, internal_client_message_receiver) = mpsc::channel(1);
        let (server_message_sender_1, mut server_message_receiver_1) = mpsc::channel(1);
        let (server_message_sender_2, mut server_message_receiver_2) = mpsc::channel(1);
        let process_internal_client_message_handle =
            async_std::task::spawn(process_internal_client_message_stream(
                internal_client_message_receiver,
                data_state,
                client_thread_state,
            ));

        // Send a Connected Message to the server
        let mut internal_client_message_sender_1 = internal_client_message_sender.clone();
        assert_eq!(
            internal_client_message_sender_1
                .send(InternalClientMessage::Connected(server_message_sender_1))
                .await,
            Ok(())
        );

        assert_eq!(
            server_message_receiver_1.next().await,
            Some(ServerMessage::YouAre(ClientId::from_count(2))),
        );

        let client_1_id = ClientId::from_count(2);

        let mut internal_client_message_sender_2 = internal_client_message_sender;
        assert_eq!(
            internal_client_message_sender_2
                .send(InternalClientMessage::Connected(server_message_sender_2))
                .await,
            Ok(()),
        );

        assert_eq!(
            server_message_receiver_2.next().await,
            Some(ServerMessage::YouAre(ClientId::from_count(3))),
        );

        assert_eq!(
            internal_client_message_sender_1
                .send(InternalClientMessage::RequestBlocksSnapshot(client_1_id))
                .await,
            Ok(()),
        );

        assert_eq!(
            server_message_receiver_1.next().await,
            Some(ServerMessage::BlocksSnapshot(Arc::new(vec![block_1]))),
        );

        // disconnect the internal client message sender
        internal_client_message_sender_1.disconnect();
        internal_client_message_sender_2.disconnect();

        // The server message receiver should be shutdown, and should return
        // nothing further
        assert_eq!(server_message_receiver_1.next().await, None);
        assert_eq!(server_message_receiver_2.next().await, None);

        // Join the async task.
        if let Err(timeout_error) = process_internal_client_message_handle
            .timeout(Duration::from_millis(200))
            .await
        {
            panic!(
                "process_internal_client_message_handle did not complete in time, error: {}",
                timeout_error
            );
        }
    }

    #[async_std::test]
    async fn test_process_client_handling_stream_request_node_identity_snapshot() {
        let (node_1, node_2, node_3, data_state) = create_test_data_state();
        let client_thread_state = Arc::new(RwLock::new(create_test_client_thread_state()));
        let data_state = Arc::new(RwLock::new(data_state));

        let (internal_client_message_sender, internal_client_message_receiver) = mpsc::channel(1);
        let (server_message_sender_1, mut server_message_receiver_1) = mpsc::channel(1);
        let (server_message_sender_2, mut server_message_receiver_2) = mpsc::channel(1);
        let process_internal_client_message_handle: async_std::task::JoinHandle<()> =
            async_std::task::spawn(process_internal_client_message_stream(
                internal_client_message_receiver,
                data_state,
                client_thread_state,
            ));

        // Send a Connected Message to the server
        let mut internal_client_message_sender_1 = internal_client_message_sender.clone();
        assert_eq!(
            internal_client_message_sender_1
                .send(InternalClientMessage::Connected(server_message_sender_1))
                .await,
            Ok(())
        );

        assert_eq!(
            server_message_receiver_1.next().await,
            Some(ServerMessage::YouAre(ClientId::from_count(2))),
        );

        let client_1_id = ClientId::from_count(2);

        // Send another Connected Message to the server
        let mut internal_client_message_sender_2 = internal_client_message_sender;
        assert_eq!(
            internal_client_message_sender_2
                .send(InternalClientMessage::Connected(server_message_sender_2))
                .await,
            Ok(())
        );

        assert_eq!(
            server_message_receiver_2.next().await,
            Some(ServerMessage::YouAre(ClientId::from_count(3))),
        );

        assert_eq!(
            internal_client_message_sender_1
                .send(InternalClientMessage::RequestNodeIdentitySnapshot(
                    client_1_id
                ))
                .await,
            Ok(()),
        );

        assert_eq!(
            server_message_receiver_1.next().await,
            Some(ServerMessage::NodeIdentitySnapshot(Arc::new(vec![
                node_1.clone(),
                node_2.clone(),
                node_3.clone()
            ]))),
        );

        // disconnect the last internal client message sender
        internal_client_message_sender_1.disconnect();
        internal_client_message_sender_2.disconnect();

        // The server message receiver should be shutdown, and should return
        // nothing further
        assert_eq!(server_message_receiver_1.next().await, None);
        assert_eq!(server_message_receiver_2.next().await, None);

        // Join the async task.
        if let Err(timeout_error) = process_internal_client_message_handle
            .timeout(Duration::from_millis(200))
            .await
        {
            panic!(
                "process_internal_client_message_handle did not complete in time, error: {}",
                timeout_error
            );
        }
    }

    #[async_std::test]
    async fn test_process_client_handling_stream_subscribe_latest_block() {
        let (_, _, _, data_state) = create_test_data_state();
        let client_thread_state = Arc::new(RwLock::new(create_test_client_thread_state()));
        let data_state = Arc::new(RwLock::new(data_state));

        let (mut leaf_sender, leaf_receiver) = mpsc::channel(1);
        let (block_detail_sender, block_detail_receiver) = mpsc::channel(1);
        let (voters_sender, voters_receiver) = mpsc::channel(1);
        let (internal_client_message_sender, internal_client_message_receiver) = mpsc::channel(1);
        let (server_message_sender_1, mut server_message_receiver_1) = mpsc::channel(1);
        let (server_message_sender_2, mut server_message_receiver_2) = mpsc::channel(1);
        let (server_message_sender_3, mut server_message_receiver_3) = mpsc::channel(1);
        let process_internal_client_message_handle =
            async_std::task::spawn(process_internal_client_message_stream(
                internal_client_message_receiver,
                data_state.clone(),
                client_thread_state.clone(),
            ));

        let process_distribute_block_detail_handle =
            async_std::task::spawn(process_distribute_block_detail_handling_stream(
                client_thread_state.clone(),
                block_detail_receiver,
            ));

        let process_distribute_voters_handle = async_std::task::spawn(
            process_distribute_voters_handling_stream(client_thread_state, voters_receiver),
        );

        let process_leaf_stream_handle = async_std::task::spawn(process_leaf_stream(
            leaf_receiver,
            data_state,
            block_detail_sender,
            voters_sender,
        ));

        // Send a Connected Message to the server
        let mut internal_client_message_sender_1 = internal_client_message_sender.clone();
        assert_eq!(
            internal_client_message_sender_1
                .send(InternalClientMessage::Connected(server_message_sender_1))
                .await,
            Ok(())
        );

        assert_eq!(
            server_message_receiver_1.next().await,
            Some(ServerMessage::YouAre(ClientId::from_count(2))),
        );

        let client_1_id = ClientId::from_count(2);
        let client_2_id = ClientId::from_count(3);

        // Send another Connected Message to the server
        let mut internal_client_message_sender_2 = internal_client_message_sender.clone();
        assert_eq!(
            internal_client_message_sender_2
                .send(InternalClientMessage::Connected(server_message_sender_2))
                .await,
            Ok(())
        );

        assert_eq!(
            server_message_receiver_2.next().await,
            Some(ServerMessage::YouAre(ClientId::from_count(3))),
        );

        // Send another Connected Message to the server
        let mut internal_client_message_sender_3 = internal_client_message_sender;
        assert_eq!(
            internal_client_message_sender_3
                .send(InternalClientMessage::Connected(server_message_sender_3))
                .await,
            Ok(())
        );

        assert_eq!(
            server_message_receiver_3.next().await,
            Some(ServerMessage::YouAre(ClientId::from_count(4))),
        );

        assert_eq!(
            internal_client_message_sender_1
                .send(InternalClientMessage::SubscribeLatestBlock(client_1_id))
                .await,
            Ok(()),
        );

        assert_eq!(
            internal_client_message_sender_1
                .send(InternalClientMessage::SubscribeLatestBlock(client_2_id))
                .await,
            Ok(()),
        );

        // No response expected from the client messages at the moment.

        // send a new leaf
        let leaf = Leaf::genesis(&ValidatedState::default(), &NodeState::mock()).await;
        let expected_block = create_block_detail_from_leaf(&leaf);
        let arc_expected_block = Arc::new(expected_block);

        assert_eq!(leaf_sender.send(leaf).await, Ok(()));

        // We should receive the Block Detail on each subscribed client
        assert_eq!(
            server_message_receiver_1.next().await,
            Some(ServerMessage::LatestBlock(arc_expected_block.clone()))
        );
        assert_eq!(
            server_message_receiver_2.next().await,
            Some(ServerMessage::LatestBlock(arc_expected_block.clone()))
        );

        // disconnect the leaf sender
        leaf_sender.disconnect();

        // Join the async task.
        if let Err(timeout_error) = process_leaf_stream_handle
            .timeout(Duration::from_millis(200))
            .await
        {
            panic!(
                "process_leaf_stream_handle did not complete in time, error: {}",
                timeout_error
            );
        }

        // Join the async task.
        if let Err(timeout_error) = process_distribute_block_detail_handle
            .timeout(Duration::from_millis(200))
            .await
        {
            panic!(
                "process_distribute_client_handling_handle did not complete in time, error: {}",
                timeout_error
            );
        }

        if let Err(timeout_error) = process_distribute_voters_handle
            .timeout(Duration::from_millis(200))
            .await
        {
            panic!(
                "process_distribute_voters_handle did not complete in time, error: {}",
                timeout_error
            );
        }

        // disconnect the last internal client message sender
        internal_client_message_sender_1.disconnect();
        internal_client_message_sender_2.disconnect();
        internal_client_message_sender_3.disconnect();

        // The server message receiver should be shutdown, and should return
        // nothing further
        assert_eq!(server_message_receiver_1.next().await, None);
        assert_eq!(server_message_receiver_2.next().await, None);
        assert_eq!(server_message_receiver_3.next().await, None);

        // Join the async task.
        if let Err(timeout_error) = process_internal_client_message_handle
            .timeout(Duration::from_millis(200))
            .await
        {
            panic!(
                "process_internal_client_message_handle did not complete in time, error: {}",
                timeout_error
            );
        }
    }

    #[async_std::test]
    async fn test_process_client_handling_stream_subscribe_node_identity() {
        let (node_1, _, _, data_state) = create_test_data_state();
        let client_thread_state = Arc::new(RwLock::new(create_test_client_thread_state()));
        let data_state = Arc::new(RwLock::new(data_state));

        let (mut node_identity_sender, node_identity_receiver) = mpsc::channel(1);
        let (internal_client_message_sender, internal_client_message_receiver) = mpsc::channel(1);
        let (server_message_sender_1, mut server_message_receiver_1) = mpsc::channel(1);
        let (server_message_sender_2, mut server_message_receiver_2) = mpsc::channel(1);
        let (server_message_sender_3, mut server_message_receiver_3) = mpsc::channel(1);
        let process_internal_client_message_handle =
            async_std::task::spawn(process_internal_client_message_stream(
                internal_client_message_receiver,
                data_state.clone(),
                client_thread_state.clone(),
            ));

        let process_distribute_node_identity_handle =
            async_std::task::spawn(process_distribute_node_identity_handling_stream(
                client_thread_state,
                node_identity_receiver,
            ));

        // Send a Connected Message to the server
        let mut internal_client_message_sender_1 = internal_client_message_sender.clone();
        assert_eq!(
            internal_client_message_sender_1
                .send(InternalClientMessage::Connected(server_message_sender_1))
                .await,
            Ok(())
        );

        assert_eq!(
            server_message_receiver_1.next().await,
            Some(ServerMessage::YouAre(ClientId::from_count(2))),
        );

        let client_1_id = ClientId::from_count(2);
        let client_2_id = ClientId::from_count(3);

        // Send another Connected Message to the server
        let mut internal_client_message_sender_2 = internal_client_message_sender.clone();
        assert_eq!(
            internal_client_message_sender_2
                .send(InternalClientMessage::Connected(server_message_sender_2))
                .await,
            Ok(())
        );

        assert_eq!(
            server_message_receiver_2.next().await,
            Some(ServerMessage::YouAre(ClientId::from_count(3))),
        );

        // Send another Connected Message to the server
        let mut internal_client_message_sender_3 = internal_client_message_sender;
        assert_eq!(
            internal_client_message_sender_3
                .send(InternalClientMessage::Connected(server_message_sender_3))
                .await,
            Ok(())
        );

        assert_eq!(
            server_message_receiver_3.next().await,
            Some(ServerMessage::YouAre(ClientId::from_count(4))),
        );

        assert_eq!(
            internal_client_message_sender_1
                .send(InternalClientMessage::SubscribeNodeIdentity(client_1_id))
                .await,
            Ok(()),
        );

        assert_eq!(
            internal_client_message_sender_1
                .send(InternalClientMessage::SubscribeNodeIdentity(client_2_id))
                .await,
            Ok(()),
        );

        // No response expected from the client messages at the moment.

        // send a new Node Identity
        let node_identity = node_1;
        assert_eq!(
            node_identity_sender.send(node_identity.clone()).await,
            Ok(())
        );

        let arc_node_identity = Arc::new(node_identity.clone());

        // We should receive the Block Detail on each subscribed client
        assert_eq!(
            server_message_receiver_1.next().await,
            Some(ServerMessage::LatestNodeIdentity(arc_node_identity.clone()))
        );
        assert_eq!(
            server_message_receiver_2.next().await,
            Some(ServerMessage::LatestNodeIdentity(arc_node_identity.clone()))
        );

        // disconnect the leaf sender
        node_identity_sender.disconnect();

        // Join the async task.
        if let Err(timeout_error) = process_distribute_node_identity_handle
            .timeout(Duration::from_millis(200))
            .await
        {
            panic!(
                "process_distribute_node_identity_handle did not complete in time, error: {}",
                timeout_error
            );
        }

        // disconnect the last internal client message sender
        internal_client_message_sender_1.disconnect();
        internal_client_message_sender_2.disconnect();
        internal_client_message_sender_3.disconnect();

        // The server message receiver should be shutdown, and should return
        // nothing further
        assert_eq!(server_message_receiver_1.next().await, None);
        assert_eq!(server_message_receiver_2.next().await, None);
        assert_eq!(server_message_receiver_3.next().await, None);

        // Join the async task.
        if let Err(timeout_error) = process_internal_client_message_handle
            .timeout(Duration::from_millis(200))
            .await
        {
            panic!(
                "process_internal_client_message_handle did not complete in time, error: {}",
                timeout_error
            );
        }
    }

    #[async_std::test]
    async fn test_process_client_handling_stream_subscribe_voters() {
        let (_, _, _, data_state) = create_test_data_state();
        let client_thread_state = Arc::new(RwLock::new(create_test_client_thread_state()));
        let data_state = Arc::new(RwLock::new(data_state));

        let (mut voters_sender, voters_receiver) = mpsc::channel(1);
        let (internal_client_message_sender, internal_client_message_receiver) = mpsc::channel(1);
        let (server_message_sender_1, mut server_message_receiver_1) = mpsc::channel(1);
        let (server_message_sender_2, mut server_message_receiver_2) = mpsc::channel(1);
        let (server_message_sender_3, mut server_message_receiver_3) = mpsc::channel(1);
        let process_internal_client_message_handle =
            async_std::task::spawn(process_internal_client_message_stream(
                internal_client_message_receiver,
                data_state.clone(),
                client_thread_state.clone(),
            ));

        let process_distribute_voters_handle = async_std::task::spawn(
            process_distribute_voters_handling_stream(client_thread_state, voters_receiver),
        );

        // Send a Connected Message to the server
        let mut internal_client_message_sender_1 = internal_client_message_sender.clone();
        assert_eq!(
            internal_client_message_sender_1
                .send(InternalClientMessage::Connected(server_message_sender_1))
                .await,
            Ok(())
        );

        assert_eq!(
            server_message_receiver_1.next().await,
            Some(ServerMessage::YouAre(ClientId::from_count(2))),
        );

        let client_1_id = ClientId::from_count(2);
        let client_2_id = ClientId::from_count(3);

        // Send another Connected Message to the server
        let mut internal_client_message_sender_2 = internal_client_message_sender.clone();
        assert_eq!(
            internal_client_message_sender_2
                .send(InternalClientMessage::Connected(server_message_sender_2))
                .await,
            Ok(())
        );

        assert_eq!(
            server_message_receiver_2.next().await,
            Some(ServerMessage::YouAre(ClientId::from_count(3))),
        );

        // Send another Connected Message to the server
        let mut internal_client_message_sender_3 = internal_client_message_sender;
        assert_eq!(
            internal_client_message_sender_3
                .send(InternalClientMessage::Connected(server_message_sender_3))
                .await,
            Ok(())
        );

        assert_eq!(
            server_message_receiver_3.next().await,
            Some(ServerMessage::YouAre(ClientId::from_count(4))),
        );

        assert_eq!(
            internal_client_message_sender_1
                .send(InternalClientMessage::SubscribeNodeIdentity(client_1_id))
                .await,
            Ok(()),
        );

        assert_eq!(
            internal_client_message_sender_1
                .send(InternalClientMessage::SubscribeNodeIdentity(client_2_id))
                .await,
            Ok(()),
        );

        // No response expected from the client messages at the moment.

        // send a new Node Identity
        let voters = BitVec::from_vec(vec![0x55]);
        assert_eq!(voters_sender.send(voters.clone()).await, Ok(()));

        // We should receive the Block Detail on each subscribed client
        assert_eq!(
            server_message_receiver_1.next().await,
            Some(ServerMessage::LatestVoters(voters.clone()))
        );
        assert_eq!(
            server_message_receiver_2.next().await,
            Some(ServerMessage::LatestVoters(voters.clone()))
        );

        // disconnect the leaf sender
        voters_sender.disconnect();

        // Join the async task.
        if let Err(timeout_error) = process_distribute_voters_handle
            .timeout(Duration::from_millis(200))
            .await
        {
            panic!(
                "process_distribute_voters_handle did not complete in time, error: {}",
                timeout_error
            );
        }

        // disconnect the last internal client message sender
        internal_client_message_sender_1.disconnect();
        internal_client_message_sender_2.disconnect();
        internal_client_message_sender_3.disconnect();

        // The server message receiver should be shutdown, and should return
        // nothing further
        assert_eq!(server_message_receiver_1.next().await, None);
        assert_eq!(server_message_receiver_2.next().await, None);
        assert_eq!(server_message_receiver_3.next().await, None);

        // Join the async task.
        if let Err(timeout_error) = process_internal_client_message_handle
            .timeout(Duration::from_millis(200))
            .await
        {
            panic!(
                "process_internal_client_message_handle did not complete in time, error: {}",
                timeout_error
            );
        }
    }

    // The following tests codify assumptions being bad on behalf of the Sender
    // and Receivers provided by the async_std library.  The purpose of these
    // tests are to document these assumptions, and add a test to ensure that
    // they behave as expected.  If they ever do not behave as expected, then
    // the rest of this library will need to be modified to account for that
    // change in behavior.

    /// Tests the behavior of the sender and receiver when the sender is
    /// dropped before the receiver is polled.
    ///
    /// This is a separate library test to ensure that the behavior that this
    /// library is built on top of does not introduce a change that would
    /// make this library no longer operate correctly.
    #[async_std::test]
    async fn test_sender_receiver_behavior_drop_sender_before_receiver_polled_closes_receiver() {
        let (sender, mut receiver) = mpsc::channel::<u64>(1);

        drop(sender);

        assert_eq!(receiver.next().await, None);
    }

    /// Tests the behavior of the  sender and receiver when the sender is
    /// dropped after the receiver is polled.
    ///
    /// This is a separate library test to ensure that the behavior that this
    /// library is built on top of does not introduce a change that would
    /// make this library no longer operate correctly.
    #[async_std::test]
    async fn test_sender_receiver_behavior_drop_sender_after_receiver_polled_closes_receiver() {
        let (sender, mut receiver) = mpsc::channel::<u64>(1);

        let join_handle = async_std::task::spawn(async move { receiver.next().await });
        async_std::task::sleep(Duration::from_millis(100)).await;
        drop(sender);

        assert_eq!(join_handle.await, None);
    }

    /// Tests the behavior of the sender and receiver when the receiver is
    /// dropped before anything is sent across the Sender.
    ///
    /// This is a separate library test to ensure that the behavior that this
    /// library is built on top of does not introduce a change that would
    /// make this library no longer operate correctly.
    #[async_std::test]
    async fn test_sender_receiver_behavior_drop_receiver_before_sender_sends() {
        let (mut sender, receiver) = mpsc::channel(1);

        drop(receiver);

        assert_ne!(sender.send(1).await, Ok(()));
    }

    /// Tests the behavior of the sender and receiver when the receiver is
    /// dropped after the sender has sent a value.
    ///
    /// This is a separate library test to ensure that the behavior that this
    /// library is built on top of does not introduce a change that would
    /// make this library no longer operate correctly.
    #[async_std::test]
    async fn test_sender_receiver_behavior_drop_receiver_after_sender_sends() {
        let (mut sender, mut receiver) = mpsc::channel(1);

        let join_handle = async_std::task::spawn(async move {
            _ = sender.send(1).await;
            async_std::task::sleep(Duration::from_millis(100)).await;
            sender.send(2).await
        });
        async_std::task::sleep(Duration::from_millis(50)).await;
        receiver.close();

        assert_eq!(receiver.next().await, Some(1));
        assert_eq!(receiver.next().await, None);
        assert_ne!(join_handle.await, Ok(()));
    }

    /// Tests to ensure that time timeout on an already ready future does not
    /// cause the future to be dropped.
    #[async_std::test]
    async fn test_timeout_on_already_ready_future() {
        assert_eq!(
            futures::future::ready(1u64).timeout(Duration::ZERO).await,
            Ok(1u64)
        );
    }

    /// Tests to ensure that time timeout on a pending future does not cause the
    /// future to be dropped.
    #[async_std::test]
    async fn test_timeout_on_async_block_resolves_when_polled() {
        assert_eq!(async move { 1u64 }.timeout(Duration::ZERO).await, Ok(1u64),);

        assert_eq!(
            async move { 1u64 }
                .timeout(Duration::from_millis(100))
                .await,
            Ok(1u64),
        );
    }

    /// Tests to ensure that time timeout on a pending future does not cause the
    /// future to be dropped.
    #[async_std::test]
    async fn test_timeout_on_pending_future_times_out() {
        assert_ne!(
            async_std::future::timeout(Duration::ZERO, futures::future::pending::<u64>()).await,
            Ok(1u64)
        );
    }

    /// Tests to ensure that bitvec is directly comparable without needing to
    /// worry about their instances points to the same memory.
    #[test]
    fn test_bitvec_is_comparable() {
        let bitvec_1: BitVec<usize> = BitVec::from_vec(vec![0x55]);
        let bitvec_2: BitVec<usize> = BitVec::from_vec(vec![0x55]);
        let bitvec_3: BitVec<usize> = BitVec::from_vec(vec![0xAA]);

        assert_eq!(bitvec_1, bitvec_2);
        assert_ne!(bitvec_1, bitvec_3);
    }
}
