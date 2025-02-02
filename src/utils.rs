//! Utility functions that perform file parsing and layout generation before
//! writing the result into a schema file.

use crate::layout::Layout;

use std::fs;
use std::io::Write;
use std::path::Path;


#[macro_export]
macro_rules! construct_layouts {
    ($($item:ident),+) => {{
        use $crate::Layout;
        use borsh::BorshSchema;

        let mut layouts = Vec::new();
        $( 
            let item_layouts =
                Layout::from_borsh_container(<$item as BorshSchema>::schema_container())
                    .unwrap();

            layouts.extend(item_layouts);
        )+
        layouts
    }};
}

static LIB_PREABMLE: &str = r#"import { BinaryReader, BinaryWriter } from "borsh";
import { PublicKey } from '@velas/web3';
import BN from "bn.js";

const borshPublicKeyHack = () => {
	// "borsh": "^0.7.0"

	// agsol-borsh-schema/test-rs-output-ts-input/node_modules/borsh/lib/index.js:258
	//             writer[`write${capitalizeFirstLetter(fieldType)}`](value);
	//                                                               ^
	// TypeError: writer[capitalizeFirstLetter(...)] is not a function
  ;(BinaryReader.prototype as any).readPublicKeyHack = function () {
    const reader = this as unknown as BinaryReader
    const array = reader.readFixedArray(32)
    return new PublicKey(array)
  }
  ;(BinaryWriter.prototype as any).writePublicKeyHack = function (value: PublicKey) {
    const writer = this as unknown as BinaryWriter
    writer.writeFixedArray(value.toBytes())
  }
}

borshPublicKeyHack();

class Struct {
  constructor(properties: any) {
    Object.keys(properties).map(key => {
      this[key as keyof typeof this] = properties[key];
    });
  }
}

class Enum {
  enum: string | undefined;

  constructor(properties: any) {
    if (Object.keys(properties).length !== 1) {
      throw new Error('Enum can only take single value');
    }
    Object.keys(properties).map(key => {
      this[key as keyof typeof this] = properties[key];
      this.enum = key;
    });
  }
}

"#;
/// Writes the generated layouts into a file in the provided output directory.
pub fn generate_output(
    layouts: &[Layout],
    output_directory: impl AsRef<Path>,
) -> Result<(), anyhow::Error> {
    let schema_string = layouts
        .iter()
        .map(|layout| layout.to_borsh_schema())
        .collect::<String>();

    let classes_string = layouts
        .iter()
        .map(|layout| layout.to_ts_class())
        .collect::<String>();


    let schema = format!(
        r#"export const SCHEMA = new Map<any, any>([{}
]);"#,
        schema_string
    );

    let imports = String::from(LIB_PREABMLE);

    fs::create_dir_all(&output_directory)?;
    let mut file = fs::File::create(output_directory.as_ref().join("schema.ts"))?;
    write!(file, "{}", imports + &classes_string + &schema)?;
    Ok(())
}
