// extern crate text_editor;
// use text_editor::{EditorInner, VecBuffer};

// #[test]
// fn test_editor_api() {
//     let editor = EditorInner::new(VecBuffer::new(generate_test_text_buffer()), true);
//     editor.move_cursor()
// }

// fn generate_test_text_buffer() -> Vec<String> {
//     let text = r#"The quick brown fox jumps over the lazy dog. \n
//         Rust provides memory safety without garbage collection. \n
//         In 1492, Columbus sailed the ocean blue. \n
//         To be or not to be, that is the question. \n
//         A journey of a thousand miles begins with a single step. \n
//         The early bird catches the worm. \n
//         All that glitters is not gold. \n
//         Actions speak louder than words. \n
//         Where there's smoke, there's fire. \n
//         Two wrongs don't make a right. \n
//         The pen is mightier than the sword. \n
//         When in Rome, do as the Romans do. \n
//         The apple doesn't fall far from the tree. \n
//         You can't judge a book by its cover. \n
//         A picture is worth a thousand words. \n
//         Necessity is the mother of invention. \n
//         A watched pot never boils. \n
//         Beggars can't be choosers. \n
//         Don't count your chickens before they hatch. \n
//         If it ain't broke, don't fix it. \n
//         The grass is always greener on the other side. \n
//         It's raining cats and dogs. \n
//         Better late than never. \n
//         You can't have your cake and eat it too. \n
//         A bird in the hand is worth two in the bush. \n
//         Rome wasn't built in a day. \n
//         Don't put all your eggs in one basket. \n
//         An apple a day keeps the doctor away. \n
//         Every cloud has a silver lining. \n
//         When it rains, it pours. \n
//         Honesty is the best policy. \n
//         Laughter is the best medicine. \n
//         Knowledge is power. \n
//         Time is money. \n
//         The customer is always right. \n
//         Practice makes perfect. \n
//         Where there's a will, there's a way. \n
//         Don't bite the hand that feeds you. \n
//         A penny saved is a penny earned. \n
//         Curiosity killed the cat. \n
//         Two heads are better than one. \n
//         The squeaky wheel gets the grease. \n
//         Money doesn't grow on trees. \n
//         When the going gets tough, the tough get going. \n
//         No pain, no gain. \n
//         Beauty is in the eye of the beholder. \n
//         Cleanliness is next to godliness. \n
//         Fortune favors the bold. \n
//         Ignorance is bliss. \n
//         Absence makes the heart grow fonder. \n
//         Don't cry over spilled milk. \n
//         The best things in life are free. \n
//         If you can't beat them, join them. \n
//         There's no place like home. \n
//         A chain is only as strong as its weakest link. \n
//         Treat others as you wish to be treated. \n
//         Variety is the spice of life. \n
//         All good things must come to an end. \n
//         The devil is in the details. \n
//         Look before you leap. \n
//         Haste makes waste. \n
//         You can lead a horse to water, but you can't make it drink. \n
//         Don't put off until tomorrow what you can do today. \n
//         A rolling stone gathers no moss. \n
//         When one door closes, another opens. \n
//         The proof is in the pudding. \n
//         Too many cooks spoil the broth. \n
//         You can't make an omelet without breaking a few eggs. \n
//         The early bird catches the worm, but the second mouse gets the cheese. \n
//         A fool and his money are soon parted. \n
//         Patience is a virtue. \n
//         The road to hell is paved with good intentions. \n
//         Slow and steady wins the race. \n
//         Don't judge a book by its cover. \n
//         A stitch in time saves nine. \n
//         Measure twice, cut once. \n
//         An ounce of prevention is worth a pound of cure. \n
//         The bigger they are, the harder they fall. \n
//         Out of sight, out of mind. \n
//         The cobbler's children have no shoes. \n
//         The exception proves the rule. \n
//         History repeats itself. \n
//         If you want something done right, do it yourself. \n
//         Jack of all trades, master of none. \n
//         A leopard can't change its spots. \n
//         The elephant in the room. \n
//         The writing is on the wall. \n
//         Barking up the wrong tree. \n
//         Close, but no cigar. \n
//         Every dog has its day. \n
//         The blind leading the blind. \n
//         Burning the candle at both ends. \n
//         Cutting corners. \n
//         The pot calling the kettle black. \n
//         Beating around the bush. \n
//         Biting off more than you can chew. \n
//         Break a leg. \n
//         Burst your bubble. \n
//         Caught red-handed. \n
//         Chasing your tail. \n
//         Comparing apples to oranges. \n
//         Don't throw the baby out with the bathwater. \n
//         Easier said than done. \n
//         Fit as a fiddle. \n
//         Get your act together. \n
//         Give credit where credit is due. \n
//         Go back to the drawing board. \n
//         Hit the nail on the head. \n
//         It's not rocket science. \n
//         Jump on the bandwagon. \n
//         Keep your chin up. \n
//         Let sleeping dogs lie. \n
//         Make a long story short. \n
//         Never say never. \n
//         On thin ice. \n
//         Once in a blue moon. \n
//         One man's trash is another man's treasure. \n
//         Pay through the nose. \n
//         Piece of cake. \n
//         Pull yourself together. \n
//         Put your money where your mouth is. \n
//         Reinvent the wheel. \n
//         Rome wasn't built in a day. \n
//         Run like the wind. \n
//         Save for a rainy day. \n
//         Stick to your guns. \n
//         Take the bull by the horns. \n
//         The last straw. \n
//         The whole nine yards. \n
//         There's no such thing as a free lunch. \n
//         Time flies when you're having fun. \n
//         To each his own. \n
//         Turn over a new leaf. \n
//         Up in the air. \n
//         We'll cross that bridge when we come to it. \n
//         What goes around comes around. \n
//         When pigs fly. \n
//         You can't teach an old dog new tricks. \n
//         A dime a dozen. \n
//         A little bird told me. \n
//         A penny for your thoughts. \n
//         Add insult to injury. \n
//         All ears. \n
//         As right as rain. \n
//         At the drop of a hat. \n
//         Back to square one. \n
//         Bark is worse than the bite. \n
//         Beat around the bush. \n
//         Bend over backwards. \n
//         Between a rock and a hard place. \n
//         Bite the bullet. \n
//         Blow off steam. \n
//         Break the ice. \n
//         Burn the midnight oil. \n
//         By the skin of your teeth. \n
//         Call it a day. \n
//         Cat got your tongue? \n
//         Champing at the bit. \n
//         Cost an arm and a leg. \n
//         Cross that bridge when you come to it. \n
//         Cry wolf. \n
//         Cut to the chase. \n
//         Don't look a gift horse in the mouth. \n
//         Down to the wire. \n
//         Drastic times call for drastic measures. \n
//         Every cloud has a silver lining. \n
//         Fall on deaf ears. \n
//         Fight fire with fire. \n
//         Foam at the mouth. \n
//         Get a taste of your own medicine. \n
//         Get wind of something. \n
//         Give the benefit of the doubt. \n
//         Go down in flames. \n
//         Go the extra mile. \n
//         Hang in there. \n
//         Happy as a clam. \n
//         Have a bone to pick. \n
//         Head over heels. \n
//         Hit the books. \n
//         Hold your horses. \n
//         In hot water. \n
//         It takes two to tango. \n
//         Jump ship. \n
//         Keep your eyes peeled. \n
//         Kick the bucket. \n
//         Kill two birds with one stone. \n
//         Knee-high to a grasshopper. \n
//         Leave no stone unturned. \n
//         Let the cat out of the bag. \n
//         Make a mountain out of a molehill. \n
//         Miss the boat. \n
//         No pain, no gain. \n
//         Off the hook. \n
//         On cloud nine. \n
//         On the ball. \n
//         Out of the woods. \n
//         Over the moon. \n
//         Piece of cake. \n
//         Pull the wool over someone's eyes. \n
//         Push the envelope. \n
//         Put all your eggs in one basket. \n
//         Rain on someone's parade. \n
//         Read between the lines. \n
//         Rise and shine. \n
//         Sail close to the wind. \n
//         See eye to eye. \n
//         Sick as a dog. \n
//         Sit on the fence. \n
//         Spill the beans. \n
//         Stand your ground. \n
//         Take it with a grain of salt. \n
//         The ball is in your court. \n
//         The best of both worlds. \n
//         Throw in the towel. \n
//         Tie the knot. \n
//         Time is of the essence. \n
//         Turn a blind eye. \n
//         Under the weather. \n
//         Up in arms. \n
//         Water under the bridge."#;
// text.lines().map(String::from).collect()
// }
