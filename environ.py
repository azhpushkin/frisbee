import zmq

c = zmq.Context()

read = c.socket(zmq.SUB)

read.bind('tcp://127.0.0.1:5557')
read.subscribe('')  # Read all topics

write = c.socket(zmq.PUB)

write.bind('tcp://127.0.0.1:5556')

import time
time.sleep(0.5)

while True:
    topic, data = read.recv_multipart()
    if topic.startswith(b'return'):
        topic = topic
    else:
        topic = b'messages:' + topic

    write.send_multipart([topic, data])

    print(f'[{topic}] {data}')



