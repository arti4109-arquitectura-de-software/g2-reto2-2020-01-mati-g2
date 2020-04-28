use crate::offers::OfferEventRequest;
use std::error::Error;
use std::sync::{Arc, Mutex};
use wapc::WapcHost;

type SerResult = Result<Vec<u8>, Box<dyn Error>>;

pub struct FormatDeserializer {
    host: Arc<Mutex<WapcHost>>,
}

unsafe impl Send for FormatDeserializer {}
unsafe impl Sync for FormatDeserializer {}

impl FormatDeserializer {
    pub fn new(wasm_path: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let module_bytes = std::fs::read(wasm_path)?;
        println!("Started Load //////////////");
        let host = WapcHost::new(
            |id: u64, bd: &str, ns: &str, op: &str, payload: &[u8]| {
                println!(
                    "Guest {} invoked '{}->{}:{}' with payload of {} bytes",
                    id,
                    bd,
                    ns,
                    op,
                    payload.len()
                );
                Ok(vec![])
            },
            &module_bytes,
            None,
        )?;
        println!("Ended Load //////////////");

        Ok(Self {
            host: Arc::new(Mutex::new(host)),
        })
    }

    pub fn deserialize(
        &self,
        header: &str,
        payload: &[u8],
    ) -> Result<OfferEventRequest, Box<dyn Error>> {
        let ans = self.host.clone().lock().unwrap().call(header, payload)?;
        Ok(bincode::deserialize::<OfferEventRequest>(&ans)?)
    }

    pub fn replace(&self, module_bytes: &[u8]) {
        let host = WapcHost::new(
            |id: u64, bd: &str, ns: &str, op: &str, payload: &[u8]| {
                println!(
                    "Guest {} invoked '{}->{}:{}' with payload of {} bytes",
                    id,
                    bd,
                    ns,
                    op,
                    payload.len()
                );
                Ok(vec![])
            },
            &module_bytes,
            None,
        ).unwrap();
        *self.host.clone().lock().unwrap() = host;
    }
}

// {
//     "type": "record",
//     "name": "TestData",
//     "fields": [
//       {"name": "f1", "type": "string"},
//       {"name": "f2", "type": ["null", "long"], "default": "null"},
//       {"name": "f3", "type": {"type":"array", "items": "int"}}
//     ]
//   }

// fn avro_format(msg: &TestData) -> SerResult {
//     let raw_schema = serde_json::json!({
//             "type": "record",
//             "name": "TestData",
//             "fields": [
//                 { "name": "type", "type": {"name": "type","type":"enum", "symbols": ["First", "Second", "Third"]} },
//                 { "name": "value", "type": [
//                     {"name":"First",  "type":{"type":"record","fields":[
//                         {"name": "f1", "type": "string"},
//                         {"name": "f2", "type": ["null", "long"], "default": "null"},
//                         {"name": "f3", "type": {"name": "f3","type":"array", "items": "int"}}
//                     ]}},
//                     {"name":"Third",
//                     "type":{"type":"record","fields":[
//                         { "name": "v1",
//                         "type":  {"name": "inner", "type":"array", "items": "string"} ,
//                         "default":"null"}
//                     ]}},
//                 ]}
//             ]
//         }
//     );

//     // let schema = avro::Schema::Union(avro::schema::UnionSchema::new(vec![])?);

//     let schema = match avro::Schema::parse(&raw_schema) {
//         Ok(e) => e,
//         Err(e) => {
//             println!("{:?}", e);
//             panic!("");
//         }
//     };
//     let mut writer = avro::Writer::new(&schema, Vec::new());

//     writer.append_ser(msg)?;
//     writer.flush()?;
//     Ok(writer.into_inner())
// }
