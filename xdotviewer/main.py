# This is very hacky but there seems to be no better way
# For the time being the latest version of xdot (that is
# required for this to work) is not available through the 
# package manager.
# We need to import the local version as a module
import sys
import pathlib
import os.path
sys.path.append(os.path.join(pathlib.Path(__file__).parent.resolve().parent.resolve().parent.resolve(), "xdot.py"))
import xdot.ui

import argparse

import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk

from chart_window import ChartWindow
from tooltip_widget import TooltipWidget
from ros_element import RosElement, NodeType, ElementType
from r2ta_interface import R2TAInterface

class MyDotWindow(xdot.ui.DotWindow):

    def __init__(self, r2ta: R2TAInterface):
        xdot.ui.DotWindow.__init__(self, widget=TooltipWidget(r2ta))
        self.r2ta = r2ta

        self.dotwidget.connect('clicked', self.on_url_clicked)

    def on_url_clicked(self, widget, url: str, event):
        def extract_r2ta_meta(url):
            element_type = None
            if url.startswith("r2ta-node://"):
                element_type = ElementType.NODE
            elif url.startswith("r2ta-edge://"):
                element_type = ElementType.EDGE
            else:
                return None, None, None
 
            node_type = None
            node_id = url[url.index("://") + 3:]
            if "interface_type=Subscriber" in node_id:
                node_type = NodeType.SUBSCRIBER
            elif "interface_type=Callback" in node_id:
                node_type = NodeType.CALLBACK
            elif "interface_type=Timer" in node_id:
                node_type = NodeType.TIMER
            elif "interface_type=Service" in node_id:
                node_type = NodeType.SERVICE
            elif "interface_type=Publisher" in node_id:
                node_type = NodeType.PUBLISHER

            try:
                return element_type, node_type, node_id
            except Exception as e:
                print("WARN: This node type is not currently supported")
                return None, None, None

        element_type, node_type, node_identifier = extract_r2ta_meta(url)


        if element_type is not None:
            window = ChartWindow(
                self.r2ta,
                RosElement(name=node_identifier, element_type=element_type, node_type=node_type)
            )

            window.show_all()

            return True
        return False


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('source_file')
    parser.add_argument('tracer_cmd')
    parser.add_argument('source_dir')
    args = parser.parse_args()

    r2ta = R2TAInterface(args.tracer_cmd, args.source_dir)
    window = MyDotWindow(r2ta)
    window.open_file(args.source_file)
    window.connect('delete-event', Gtk.main_quit)
    Gtk.main()


if __name__ == '__main__':
    main()
