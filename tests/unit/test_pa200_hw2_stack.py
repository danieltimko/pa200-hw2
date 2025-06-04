import aws_cdk as core
import aws_cdk.assertions as assertions

from pa200_hw2.pa200_hw2_stack import Pa200Hw2Stack

# example tests. To run these tests, uncomment this file along with the example
# resource in pa200_hw2/pa200_hw2_stack.py
def test_sqs_queue_created():
    app = core.App()
    stack = Pa200Hw2Stack(app, "pa200-hw2")
    template = assertions.Template.from_stack(stack)

#     template.has_resource_properties("AWS::SQS::Queue", {
#         "VisibilityTimeout": 300
#     })
