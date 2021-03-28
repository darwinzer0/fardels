import React, { Component } from 'react';
import { connect } from 'react-redux';
import Resizer from 'react-image-file-resizer';
import Autolinker from 'autolinker';

//Material-UI
import Box from '@material-ui/core/Box';
import Button from '@material-ui/core/Button';
import Grid from '@material-ui/core/Grid';
import Paper from '@material-ui/core/Paper';
import Typography from '@material-ui/core/Typography';
import IconButton from '@material-ui/core/IconButton';
import PhotoCamera from '@material-ui/icons/PhotoCamera';
import TextField from '@material-ui/core/TextField';
import InputAdornment from '@material-ui/core/InputAdornment';
import { withStyles } from '@material-ui/core/styles';

import { 
         executeRegister,
         executeSetViewingKey,
         setHandle, 
         setProfileDescription, 
         setProfileThumbnail } from './store/Actions';

const useStyles = theme => ({
  button: {
    margin: theme.spacing(1),
  },
  buttonBox: {
    width: '100%',
  },
  root: {
    display: 'flex',
    flexWrap: 'wrap',
  },
  textFieldContents: {
    margin: theme.spacing(1),
  },
  textFieldPrice: {
    margin: theme.spacing(1),
    width: '25ch',
  },
  divider: {
    height: '1px',
    marginLeft: '20px',
    marginRight: '20px',
  },
  heading: {
    fontSize: theme.typography.pxToRem(15),
    fontWeight: theme.typography.fontWeightRegular,
  },
  paper: {
    margin: 8,
    padding: theme.spacing(3),
  },
});

class Profile extends Component {

  constructor(props) {
    super(props);
    this.thumbnailChangedHandler = this.thumbnailChangedHandler.bind(this);
    this.state = { newViewingKey: "" };
  }

  thumbnailChangedHandler(event) {
    const { setProfileThumbnail } = this.props;

    let fileInput = false
    if (event.target.files[0]) {
      fileInput = true
    }
    if (fileInput) {
      try {
        Resizer.imageFileResizer(
          event.target.files[0],
          100,
          100,
          'JPEG',
          100,
          0,
          uri => {
            setProfileThumbnail(uri)
          },
          'base64',
          null,
          null,
        );
      } catch(err) {
        console.log(err)
      }
    }
  }

  handleSaveThumbnailButtonClick() {
    console.log("save thumbnail");
  }

  handleSaveProfileButtonClick() {
    const { executeRegister, handle, profileDescription, } = this.props;
    executeRegister(handle, profileDescription);
  }

  handleInputHandler(event) {
    const { setHandle, handle } = this.props;
    
    if (((new TextEncoder().encode(event.target.value))).length <= 64) {
      setHandle(event.target.value);
    } else {
      setHandle(handle);
    }
  }

  profileDescriptionInputHandler(event) {
    const { setProfileDescription, profileDescription } = this.props;
    
    if (((new TextEncoder().encode(event.target.value))).length <= 280) {
      setProfileDescription(event.target.value);
    } else {
      setProfileDescription(profileDescription);
    }
  }

  setViewingKeyInputHandler(event) {
    this.setState({newViewingKey: event.target.value});
  }

  handleSetViewingKeyButtonClick() {
    const { executeSetViewingKey } = this.props;
    executeSetViewingKey(this.state.newViewingKey);
    this.setState({newViewingKey: ""});
  }

  render() {
    const { classes, handle, profileDescription, profileThumbnail, } = this.props;

    return (
      <div className={classes.root}>
        <Grid container spacing={1} alignItems={"center"}>
          <Grid item xs={8}>
            <TextField 
              id="profile-handle" 
              className={classes.textFieldContents}
              label="Handle"
              fullWidth
              margin="normal"
              value={handle}
              InputProps={{
                startAdornment: <InputAdornment position="start">{"@"}</InputAdornment>,
              }}
              onInput={(e) => this.handleInputHandler(e) }
            />
          </Grid>
          <Grid item xs={1}>
            <input
              accept="image/*"
              className={classes.input}
              style={{ display: 'none' }}
              id="raised-button-file"
              multiple
              type="file"
              onChange={this.thumbnailChangedHandler}
            />
            <label htmlFor="raised-button-file">
              <IconButton color="primary" aria-label="New thumbnail" component="span">
                <PhotoCamera />
              </IconButton>
            </label>
          </Grid>
          <Grid item xs={3}>
              <img src={profileThumbnail} alt='' />
          </Grid>
          <Grid item xs={8}>
            <TextField
              id="profile-description"
              className={classes.textFieldContents}
              label="Description"
              value={profileDescription}
              fullWidth
              multiline
              rowsMax={4}
              margin="normal"
              onInput={(e) => this.profileDescriptionInputHandler(e) }
            />
          </Grid>
          <Grid item xs={4} />
          <Grid item xs={8}>
            <Box className={classes.buttonBox} display="flex" justifyContent="flex-start" p={1}>
              <Button
                variant="contained"
                color="default"
                className={classes.button}
                onClick={() => this.handleSaveProfileButtonClick()}
              >
                Save Profile
              </Button>
            </Box>
          </Grid>
          <Grid item xs={4}>
            <Box className={classes.buttonBox} display="flex" justifyContent="flex-start" p={1}>
              <Button
                variant="contained"
                color="default"
                className={classes.button}
                onClick={() => this.handleSaveThumbnailButtonClick()}
              >
                Save Thumbnail
              </Button>
            </Box>
          </Grid>
          <Grid item xs={6}>
            <TextField
              id="set-viewing-key"
              className={classes.textFieldContents}
              label="Set viewing key"
              value={this.state.newViewingKey}
              fullWidth
              margin="normal"
              type="password"
              onInput={(e) => this.setViewingKeyInputHandler(e) }
            />
            <Box className={classes.buttonBox} display="flex" justifyContent="flex-start" p={1}>
              <Button
                variant="contained"
                color="default"
                className={classes.button}
                disabled={this.state.newViewingKey.length === 0}
                onClick={() => this.handleSetViewingKeyButtonClick()}
              >
                Set Viewing Key
              </Button>
            </Box>
          </Grid>
        </Grid>
      </div>
    );
  }
}

const mapStateToProps = (state) => {
  return {
    keplrEnabled: state.keplrEnabled,
    handle: state.handle,
    profileThumbnail: state.profileThumbnail,
    profileDescription: state.profileDescription,
  }
};

const mapDispatchToProps = (dispatch) => {
  return ({
    executeRegister: (handle, description) => { dispatch(executeRegister(handle, description)) },
    executeSetViewingKey: (key) => { dispatch(executeSetViewingKey(key)) },
    setHandle: (handle) => { dispatch(setHandle(handle)) },
    setProfileDescription: (description) => { dispatch(setProfileDescription(description)) },
    setProfileThumbnail: (thumbnail) => { dispatch(setProfileThumbnail(thumbnail)) },
  });
}

export default connect(mapStateToProps, mapDispatchToProps)(withStyles(useStyles)(Profile));
