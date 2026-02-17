from dataclasses import dataclass
from enum import Enum

class ChartValue(Enum):
    CALLBACK_DURATION = 'callback-duration'
    MESSAGE_LATENCY = 'messages-latency'
    ACTIVATIONS_DELAY = 'activations-delay'
    MESSAGES_DELAY = "messages-delay"
    PUBLICATIONS_DELAY = 'publications-delay'

class ChartType(Enum):
    HISTOGRAM = 'histogram'
    SCATTER = 'scatter'

class ElementType(Enum):
    NODE = 'node'
    EDGE = 'edge'

class NodeType(Enum):
    CALLBACK = 'callback'
    TIMER = 'timer'
    SERVICE = 'service'
    PUBLISHER = 'publisher'
    SUBSCRIBER = 'subscriber'

@dataclass
class ChartRequest:
    node: str
    value: ChartValue
    plot: ChartType
    bins: int | None = None
    size: int = 800

@dataclass
class RosElement:
    name: str
    element_type: ElementType
    node_type: NodeType
