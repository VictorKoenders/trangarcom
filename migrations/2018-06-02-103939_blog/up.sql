-- Your SQL goes here
CREATE TABLE BlogPost (
    id UUID PRIMARY KEY DEFAULT (uuid_generate_v4()),
    "date" DATE NOT NULL,
    published BOOLEAN NOT NULL,
    seo_name TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    content TEXT NOT NULL,
    tweet_id TEXT NULL
);

INSERT INTO BlogPost ("date", published, seo_name, title, summary, content, tweet_id)
VALUES (
    '2018-04-23',
    true,
    'neural_network_on_micro_controller',
    'Neural networks on a micro controller - Preparation and math',
    'I''ve been playing around with the idea of running a neural network on a micro controller.

This blog describes the initial idea and does some calculations on the viability of the project.',
    'I''ve been playing around with the idea of running a neural network on a micro controller.

This blog post expects the reader to have knowledge of neural networks. For more information, you can check out the following links:
* [http://neuralnetworksanddeeplearning.com/](http://neuralnetworksanddeeplearning.com/)
* [http://www.deeplearningbook.org/contents/intro.html](http://www.deeplearningbook.org/contents/intro.html)
* [https://uk.mathworks.com/help/nnet/ug/improve-neural-network-generalization-and-avoid-overfitting.html?w.mathworks.com](https://uk.mathworks.com/help/nnet/ug/improve-neural-network-generalization-and-avoid-overfitting.html?w.mathworks.com)

This project is not designed to be the most efficient neural network, but rather one that runs on a micro controller with very limited resources.
The goal
The goal is to make a robot on the end of a stick. The robot will have several motorized joints and potentiometers, and will be attached to a center point with a long stick. The center point also has a rotary encoder which measures the angle.

The robot will have a single [blue pill](http://wiki.stm32duino.com/index.php?title=Blue_Pill) controlling it. The micro controller has the following specs:
* 72 MHz clock speed
* 64 KB flash
* 20 KB RAM
The micro controller will run [Rust](https://www.rust-lang.org/en-US/) firmware, using the [stm32f103xx-hal](https://github.com/japaric/stm32f103xx-hal) crate.

The neural network is a modification of my rust_neural_network crate, modified to run without Rust''s standard library, suitable for running on a micro controller. This spawned this [no_std branch](https://github.com/victorkoenders/rust_neural_network/tree/no_std).

A central controller (Arduino or Raspberry Pi) will turn the robot on for 30 seconds. In those 30 seconds, the robot has to make as many laps as possible, based on its own sensors. At the end of the 30 seconds, the central controller will:
* Reset the physical orientation of the robot, by lifting it off the ground and letting gravity do its work
* Signal the micro controller that it needs to end the current neural network.
* Signal the micro controller how far it''s gotten.
* Wait for the micro controller to be ready for the next test.
# Memory math fun
The first concern with running a neural network on a micro controller is simple: there is no memory allocator. This can be solved 2 ways:
1. Using an existing memory allocator, or writing my own. This is possible, however we''re going to run into RAM issues later, so this seems like overhead we cannot afford to have.
2. Don''t use a memory allocator, and put everything on the Stack.

Initially both options are a possibility, but then we run into the second issue:

Neural networks are big.

Putting some numbers in an excel sheet gives me the following:

![Spreadsheet](/images/blog_nn_spreadsheet.PNG)

Having a neural network, with 8 input nodes, 8 output nodes, and a 10x10 node grid, and every value being a 4-byte float, would be around 4676 bytes, or almost 4.6 KB.

Because the neural network of choice is an evolutionary one, we also need multiple networks. Each network would be evaluated, and the best neural network would be picked to mate with a random one. One of the worst ones would randomly be removed.

That means if we have 10 neural networks in memory, we''d already be using 46 KB of memory. Our micro controller only has 20 KB ram.

Clearly this is a problem.

But there are some solutions to this:
1. Make the neural networks smaller. Meh
2. Add external RAM. Possible, but this seems difficult
3. Use a library that adds 16-bit floats. Yes please.
4. Write the inactive neural networks to flash / EEPROM. Bingo.

Applying optimizations 3 and 4, we can do much better. Storing 10 neural networks in flash allows us to use 64 KB / 10 ≈ 6000 bytes of data (we still need to store our program somewhere). Let''s drop this into our spreadsheet.

![Spreadsheet 2](/images/blog_nn_spreadsheet_2.PNG)

14x14 is 196 nodes, that''s almost doubled our initial network size! And now we store an additional 9 in our flash memory to be read in later.
# CPU speed
There is one number left that was mentioned with the introduction of the micro controller; 72 MHz clock speed. This means that the micro controller runs on 72 000 000 cycles per second, or roughly 14 nanoseconds per cycle.

Floating calculations are expensive. In addition, with the usage of an f16 library, we will not have support for hardware accelerated float calculation.

Every time the neural network does a calculation, every single node has to be calculated. This sounds like a rough calculation but we can make a very simple estimate: every 2 bytes has to be calculated once. 

This means we''ll roughly need to do 3 000 calculations for a neural network to parse a single set of inputs.

The system does not have to be particularly fast. If it can calculate a value 10 times a second, this is more than fast enough. That still leaves 7.2 million cycles to do 3 000 calculations, or 2400 cycles per calculation. This number seems high enough that the CPU speed does not seem an issue.

*Insert flash forward here where CPU speed DOES become an issue and I have to eat my words*

# The plan
So the plan is as follows:
1. Write 10 neural networks to the flash memory.
2. Load the first neural network (sized at 4.6 KB) into memory.
3. Run for 30 seconds.
4. Get the value of the current attempt and store it somewhere.
5. The micro controller loads the next neural network into memory.
6. Once all neural networks are done:
	1. pick 2 neural networks to merge (A and B)
	2. Pick a neural network to overwrite (C)
	3. Simply pick random f16 between network A and network B, and write that to the same location in C.
7. Start over

Time to get coding!

# Further optimizations
If the plan above is not sufficient enough, it is always possible to add an [SD card](https://www.google.nl/search?q=spi%20sd%20card) reader, which would allow me to store several GB of data structures. This would allow extensive logging of other data, like all the inputs of all sensors, or every iteration of every neural network. The limitation of 20 KB of memory would still exist, so this cannot scale up the network infinitely.

To get over the memory limit, there is a concept called an [External Memory Interface](https://en.wikipedia.org/wiki/External_memory_interface). Currently it is unknown if the stm32f103 supports this feature.',
    '999532792034873344'
)