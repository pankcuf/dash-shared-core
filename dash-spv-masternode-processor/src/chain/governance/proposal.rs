use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProposalJson {
    #[serde(rename = "name")]
    pub identifier: String,
    pub url: String,
    pub payment_address: String,
    // todo: make decimal conversion
    //[[[NSDecimalNumber decimalNumberWithDecimal:[proposalDictionary[@"payment_amount"] decimalValue]] decimalNumberByMultiplyingByPowerOf10:8] unsignedLongLongValue];
    pub payment_amount: u64,
    pub start_epoch: u64,
    pub end_epoch: u64,


    // NSString *identifier = nil;
    // uint64_t amount = 0;
    // uint64_t startEpoch = 0;
    // uint64_t endEpoch = 0;
    // NSString *paymentAddress = nil;
    // NSString *url = nil;
    //
    // identifier = proposalDictionary[@"name"];
    // startEpoch = [proposalDictionary[@"start_epoch"] longLongValue];
    // endEpoch = [proposalDictionary[@"end_epoch"] longLongValue];
    // paymentAddress = proposalDictionary[@"payment_address"];
    // amount = [[[NSDecimalNumber decimalNumberWithDecimal:[proposalDictionary[@"payment_amount"] decimalValue]] decimalNumberByMultiplyingByPowerOf10:8] unsignedLongLongValue];
    // url = proposalDictionary[@"url"];

}

// id governanceArray = [NSJSONSerialization JSONObjectWithData:governanceMessageData options:0 error:&jsonError];
// NSDictionary *proposalDictionary = [governanceArray isKindOfClass:[NSDictionary class]] ? governanceArray : nil;
// while (!proposalDictionary) {
//     if ([governanceArray count]) {
//         if ([governanceArray count] > 1 && [[governanceArray objectAtIndex:0] isEqualToString:@"proposal"]) {
//             proposalDictionary = [governanceArray objectAtIndex:1];
//         } else if ([[governanceArray objectAtIndex:0] isKindOfClass:[NSArray class]]) {
//             governanceArray = [governanceArray objectAtIndex:0];
//         } else if ([[governanceArray objectAtIndex:0] isKindOfClass:[NSDictionary class]]) {
//             proposalDictionary = [governanceArray objectAtIndex:0];
//         } else {
//             break;
//         }
//     } else {
//         break;
//     }
// }
// if (proposalDictionary) {
//     identifier = proposalDictionary[@"name"];
//     startEpoch = [proposalDictionary[@"start_epoch"] longLongValue];
//     endEpoch = [proposalDictionary[@"end_epoch"] longLongValue];
//     paymentAddress = proposalDictionary[@"payment_address"];
//     amount = [[[NSDecimalNumber decimalNumberWithDecimal:[proposalDictionary[@"payment_amount"] decimalValue]] decimalNumberByMultiplyingByPowerOf10:8] unsignedLongLongValue];
//     url = proposalDictionary[@"url"];
// }
