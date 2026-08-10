/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;

use profile_traits::mem::ProfilerChan as MemProfilerChan;
use servo_base::generic_channel::GenericSender;
use storage_traits::StorageThreads;
use storage_traits::client_storage::ClientStorageThreadHandle;
use storage_traits::indexeddb::IndexedDBThreadMsg;
use storage_traits::webstorage_thread::WebStorageThreadMsg;

use crate::{ClientStorageThreadFactory, IndexedDBThreadFactory, WebStorageThreadFactory};

fn new_storage_thread_group(
    mem_profiler_chan: MemProfilerChan,
    config_dir: Option<PathBuf>,
    temporary_storage: bool,
    label: &str,
) -> StorageThreads {
    let client_storage: ClientStorageThreadHandle =
        ClientStorageThreadFactory::new(config_dir.clone(), temporary_storage);
    let idb: GenericSender<IndexedDBThreadMsg> = IndexedDBThreadFactory::new(
        mem_profiler_chan.clone(),
        format!("indexedDB-reporter-{label}"),
    );
    let web_storage: GenericSender<WebStorageThreadMsg> = WebStorageThreadFactory::new(
        config_dir,
        mem_profiler_chan,
        format!("storage-reporter-{label}"),
    );

    StorageThreads::new(client_storage.into(), idb, web_storage)
}

fn disabled_storage_thread_group() -> StorageThreads {
    let (client_storage, client_storage_receiver) = servo_base::generic_channel::channel().unwrap();
    let (idb, idb_receiver) = servo_base::generic_channel::channel().unwrap();
    let (web_storage, web_storage_receiver) = servo_base::generic_channel::channel().unwrap();
    drop((client_storage_receiver, idb_receiver, web_storage_receiver));
    StorageThreads::new(client_storage, idb, web_storage)
}

pub fn new_storage_threads(
    mem_profiler_chan: MemProfilerChan,
    config_dir: Option<PathBuf>,
    temporary_storage: bool,
    disabled: bool,
) -> (StorageThreads, StorageThreads) {
    if disabled {
        return (
            disabled_storage_thread_group(),
            disabled_storage_thread_group(),
        );
    }
    let private_storage_threads = new_storage_thread_group(
        mem_profiler_chan.clone(),
        config_dir.clone(),
        temporary_storage,
        "private",
    );
    let public_storage_threads =
        new_storage_thread_group(mem_profiler_chan, config_dir, temporary_storage, "public");

    (private_storage_threads, public_storage_threads)
}
