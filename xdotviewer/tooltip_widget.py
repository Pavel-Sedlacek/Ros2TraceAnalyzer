import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk

import xdot.ui

from r2ta_interface import R2TAInterface
from ros_element import ChartRequest, ChartValue, ChartType, ElementType

class TooltipWidget(xdot.ui.DotWidget):
    from xdot.ui.actions import TooltipContext as tooltip

    def __init__(self, r2ta: R2TAInterface):
        xdot.ui.DotWidget.__init__(self)
        self.r2ta = r2ta
        self.prev_element = None
        self.prev_element_managed = False

        self.tooltip.add_widget("tooltip_image", Gtk.Image())

    def on_hover(self, element, _):
        def get_rt2a_tooltip(element):
            if hasattr(element, "tooltip") and element.tooltip.startswith("r2ta-node://"):
                return element.tooltip.removeprefix("r2ta-node://").strip(), ElementType.NODE
            if hasattr(element, "tooltip") and element.tooltip.startswith("r2ta-edge://"):
                return element.tooltip.removeprefix("r2ta-edge://").strip(), ElementType.EDGE
            return None, None

        if self.prev_element != element:
            self.prev_element = element
            self.tooltip.reset()
            
            tooltip, element_type = get_rt2a_tooltip(element)
            if tooltip is not None:
                if element_type == ElementType.NODE:
                    if "interface_type=Subscriber" in tooltip:
                        chart_value = ChartValue.MESSAGES_DELAY
                    elif "interface_type=Callback" in tooltip:
                        chart_value = ChartValue.CALLBACK_DURATION
                    elif "interface_type=Timer" in tooltip:
                        chart_value = ChartValue.ACTIVATIONS_DELAY
                    elif "interface_type=Publisher" in tooltip:
                        chart_value = ChartValue.PUBLICATIONS_DELAY
                elif element_type == ElementType.EDGE:
                    chart_value = ChartValue.MESSAGE_LATENCY

                image_path = self.r2ta.render(ChartRequest(node=tooltip.strip(), value=chart_value, plot=ChartType.HISTOGRAM, size=400))

                image = self.tooltip.get_widget("tooltip_image")
                image.set_from_file(image_path)
                image.show()
                
            self.prev_element_managed = tooltip is not None

        return self.prev_element_managed