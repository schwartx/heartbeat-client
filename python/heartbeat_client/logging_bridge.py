"""Bridge standard logging to loguru."""

import logging
from typing import override

from loguru import logger


class InterceptHandler(logging.Handler):
    """Handler that redirects standard logging to loguru."""

    @override
    def emit(self, record: logging.LogRecord) -> None:
        """
        Emit a log record to loguru.

        Parameters
        ----------
        record : logging.LogRecord
            The log record to emit
        """
        try:
            level = logger.level(record.levelname).name
        except ValueError:
            level = record.levelno

        frame = logging.currentframe()
        depth = 2
        while frame and frame.f_code.co_filename == logging.__file__:
            frame = frame.f_back
            depth += 1

        logger.opt(depth=depth, exception=record.exc_info).log(
            level, record.getMessage()
        )


def setup_logging_bridge() -> None:
    """
    Configure standard logging to redirect all logs to loguru.

    This function intercepts all standard logging calls and redirects them
    to loguru, ensuring consistent logging throughout the application.

    Examples
    --------
    >>> from world.logging_bridge import setup_logging_bridge
    >>> setup_logging_bridge()
    >>> import logging
    >>> logging.info("This will be logged via loguru")
    """
    # Configure root logger
    logging.basicConfig(handlers=[InterceptHandler()], level=0, force=True)

    # Remove all handlers from existing loggers and enable propagation
    for name in logging.root.manager.loggerDict:
        logging.getLogger(name).handlers = []
        logging.getLogger(name).propagate = True
