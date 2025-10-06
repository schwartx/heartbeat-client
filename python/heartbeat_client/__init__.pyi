from .logging_bridge import InterceptHandler as InterceptHandler
from .logging_bridge import setup_logging_bridge as setup_logging_bridge

class HeartbeatClient:
    """WebSocket heartbeat client for maintaining persistent connections."""

    def __init__(
        self,
        server_host: str = "chan.local",
        server_port: int = 14999,
        heartbeat_interval: int = 60,
        channel_id: str = "gp_factory",
        timeout_message: str = "gp_factory可能已停止运行(客户端: {client_address}) ",
        reconnect_interval: int = 60,
    ) -> None:
        """
        Initialize a heartbeat client.

        Parameters
        ----------
        server_host : str, default="chan.local"
            WebSocket server hostname
        server_port : int, default=14999
            WebSocket server port
        heartbeat_interval : int, default=60
            Interval between heartbeats in seconds
        channel_id : str, default="gp_factory"
            Channel identifier for registration
        timeout_message : str, default="gp_factory可能已停止运行(客户端: {client_address}) "
            Message to send on timeout
        reconnect_interval : int, default=60
            Maximum reconnect backoff interval in seconds
        """
        ...

    async def start(self) -> None:
        """
        Start the heartbeat loop in the background.

        This is an async method and must be awaited.
        """
        ...

    def stop(self) -> None:
        """Stop the heartbeat loop."""
        ...

__all__ = ["HeartbeatClient", "InterceptHandler", "setup_logging_bridge"]
