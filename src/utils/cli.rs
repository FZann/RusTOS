//! RusTOS - Rust Real Time Operating System 
//! Copyright (C) 2025 - Fabio Zanin - fabio.zanin93@outlook.com
//! 
//! This program is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License.
//! 
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU General Public License for more details.
//! 
//! You should have received a copy of the GNU General Public License
//! along with this program.  If not, see <https://www.gnu.org/licenses/>.

/*
 * *************** COMMAND LINE INTERFACE ***************
 * 
 * Interfaccia Seriale (UART, USB, SPI...) per il controllo di RusTOS
 *
 * Il modulo esporta il task della console e include tutte le
 * funzionalità necessarie per la gestione dei comandi.
 * 
 * TODO: task table accessibile da console
 * 
 */

use core::cell::Cell;

use crate::drivers::serial::SerialStream;


const SPLASH: &str =  r" ____           _____ ___  ____  
                        |  _ \ _   _ __|_   _/ _ \/ ___| 
                        | |_) | | | / __|| || | | \___ \ 
                        |  _ <| |_| \__ \| || |_| |___) |
                        |_| \_\\__,_|___/|_| \___/|____/ ";




pub struct Console<'sp> {
    rx: Cell<[char; 128]>,
    tx: Cell<[char; 128]>,
    port: &'sp dyn SerialStream
}

impl<'sp> Console<'sp> {
    const EMPTY_BUFF: [char; 128] = ['0'; 128];

    pub fn new(port: &'sp dyn SerialStream) -> Self {
        Console {
            port,
            rx: Cell::new(Self::EMPTY_BUFF),
            tx: Cell::new(Self::EMPTY_BUFF),
        }
    }

    #[inline]
    pub fn print(&self, str: &str) {

    }

    #[inline]
    pub fn splash(&self) {
        self.print(SPLASH);
    }

    pub fn prompt(&self, path: &str) {
        let buffer = self.rx.get();
        
        self.print("root@RusTOS:");
        self.print("");
        self.print("$ ");
    }

    pub fn read(&self) {
        self.rx.replace(Self::EMPTY_BUFF);
    }

    pub fn readn(&self, num: usize) {
        self.rx.replace(Self::EMPTY_BUFF);


    }

}