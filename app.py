#!/usr/bin/env python3

import aws_cdk as cdk

from pa200_hw2_stack import Hw2Stack


app = cdk.App()
Hw2Stack(app, "Hw2Stack")

app.synth()
